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
    let client_id = match env::var("GITHUB_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => return Html(config_error_html("GITHUB_CLIENT_ID")).into_response(),
    };
    let redirect_url = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/callback".to_string());

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=public_repo",
        client_id, redirect_url
    );

    Redirect::to(&auth_url).into_response()
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

async fn github_callback(Query(query): Query<AuthRequest>) -> impl IntoResponse {
    let client_id = match env::var("GITHUB_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => return Html(config_error_html("GITHUB_CLIENT_ID")).into_response(),
    };
    let client_secret = match env::var("GITHUB_CLIENT_SECRET") {
        Ok(v) => v,
        Err(_) => return Html(config_error_html("GITHUB_CLIENT_SECRET")).into_response(),
    };
    let redirect_url = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/callback".to_string());

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", query.code.as_str()),
        ("redirect_uri", redirect_url.as_str()),
    ];

    let http_client = reqwest::Client::new();
    let res = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
                    let html = format!(
                        r#"<!DOCTYPE html>
<html>
<head><title>Authentication Successful</title></head>
<body>
    <script>
        window.opener.postMessage({{ type: 'github-auth', token: '{}' }}, '*');
        window.close();
    </script>
    <p>Authentication successful! You can close this window.</p>
</body>
</html>"#,
                        token
                    );
                    return Html(html).into_response();
                }

                // GitHub returned an error field (bad code, expired, etc.)
                let error_msg = json
                    .get("error_description")
                    .and_then(|e| e.as_str())
                    .or_else(|| json.get("error").and_then(|e| e.as_str()))
                    .unwrap_or("Unknown error from GitHub");
                eprintln!("GitHub OAuth error: {}", error_msg);
                return Html(auth_error_html(error_msg)).into_response();
            }
        }
        Err(e) => {
            eprintln!("OAuth exchange failed: {:?}", e);
        }
    }

    Html(auth_error_html("OAuth exchange failed — check server logs")).into_response()
}

// ─── Error pages ─────────────────────────────────────────────────────────────

fn config_error_html(var_name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Server Configuration Error</title></head>
<body>
    <script>
        window.opener && window.opener.postMessage(
            {{ type: 'github-auth-error', error: 'Server misconfigured: {var_name} is not set' }},
            '*'
        );
    </script>
    <p style="font-family:sans-serif;color:#c00">
        <strong>Server configuration error:</strong><br>
        <code>{var_name}</code> is not set.<br><br>
        Add it to a <code>.env</code> file in the <code>compiler/</code> directory
        or export it before starting the server.
    </p>
</body>
</html>"#
    )
}

fn auth_error_html(msg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Authentication Failed</title></head>
<body>
    <script>
        window.opener && window.opener.postMessage(
            {{ type: 'github-auth-error', error: '{msg}' }},
            '*'
        );
        window.close();
    </script>
    <p style="font-family:sans-serif;color:#c00">Authentication failed: {msg}</p>
</body>
</html>"#
    )
}
