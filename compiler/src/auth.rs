use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::env;

pub fn auth_router() -> Router {
    Router::new()
        .route("/github", get(github_auth))
        .route("/callback", get(github_callback))
}

async fn github_auth() -> impl IntoResponse {
    let client_id = env::var("GITHUB_CLIENT_ID").expect("Missing GITHUB_CLIENT_ID");
    let redirect_url = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/callback".to_string());
    
    // CSRF state could be added here
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=public_repo",
        client_id, redirect_url
    );

    Redirect::to(&auth_url)
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

async fn github_callback(Query(query): Query<AuthRequest>) -> impl IntoResponse {
    let client_id = env::var("GITHUB_CLIENT_ID").expect("Missing GITHUB_CLIENT_ID");
    let client_secret = env::var("GITHUB_CLIENT_SECRET").expect("Missing GITHUB_CLIENT_SECRET");
    let redirect_url = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/callback".to_string());

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", query.code.as_str()),
        ("redirect_uri", redirect_url.as_str()),
    ];

    let http_client = reqwest::Client::new();
    let res = http_client.post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
                    let html = format!(
                        r#"
<!DOCTYPE html>
<html>
<head><title>Authentication Successful</title></head>
<body>
    <script>
        window.opener.postMessage({{ type: 'github-auth', token: '{}' }}, '*');
        window.close();
    </script>
    <p>Authentication successful! You can close this window.</p>
</body>
</html>
"#,
                        token
                    );
                    return Html(html).into_response();
                }
            }
        }
        Err(e) => {
            eprintln!("OAuth exchange failed: {:?}", e);
        }
    }

    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Authentication Failed</title></head>
<body>
    <script>
        window.opener.postMessage({ type: 'github-auth-error', error: 'OAuth exchange failed' }, '*');
        window.close();
    </script>
    <p>Authentication failed. You can close this window.</p>
</body>
</html>
"#;
    Html(html).into_response()
}
