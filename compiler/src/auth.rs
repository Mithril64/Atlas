use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;

pub fn auth_router() -> Router {
    Router::new()
        .route("/github", get(github_auth))
        .route("/callback", get(github_callback))
        .route("/profile", get(get_profile))
}

fn github_authorize_url(client_id: &str, redirect_url: &str) -> String {
    format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=repo",
        client_id, redirect_url
    )
}

async fn github_auth() -> impl IntoResponse {
    let client_id = match env::var("GITHUB_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => return Html(config_error_html("GITHUB_CLIENT_ID")).into_response(),
    };
    let redirect_url = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/callback".to_string());

    let auth_url = github_authorize_url(&client_id, &redirect_url);

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
                    
                    let client = reqwest::Client::new();
                    if let Ok(user_res) = client.get("https://api.github.com/user")
                        .header("Authorization", format!("Bearer {}", token))
                        .header("User-Agent", "atlas-bot")
                        .header("Accept", "application/json")
                        .send().await {
                        if let Ok(user) = user_res.json::<serde_json::Value>().await {
                            if let (Some(id), Some(login)) = (user["id"].as_i64(), user["login"].as_str()) {
                                let id_str = id.to_string();
                                let avatar = user["avatar_url"].as_str().unwrap_or("");
                                
                                if let Ok(conn) = rusqlite::Connection::open("atlas.db") {
                                    let _ = conn.execute(
                                        "INSERT INTO users (github_id, username, avatar_url) VALUES (?1, ?2, ?3)
                                         ON CONFLICT(github_id) DO UPDATE SET
                                            username = excluded.username,
                                            avatar_url = excluded.avatar_url",
                                        [&id_str, login, avatar]
                                    );
                                }
                            }
                        }
                    }

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

#[derive(Debug, Serialize)]
pub struct ContributionDay {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub github_id: String,
    pub username: String,
    pub avatar_url: String,
    pub commits: i64,
    pub reviews: i64,
    pub trust_rating: i64,
    pub contribution_days: Vec<ContributionDay>,
}

pub async fn get_profile(headers: axum::http::HeaderMap) -> impl IntoResponse {
    let auth_token = headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    if let Some(token) = auth_token {
        let client = reqwest::Client::new();
        if let Ok(res) = client.get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "atlas-bot")
            .header("Accept", "application/json")
            .send().await {
            if let Ok(user) = res.json::<serde_json::Value>().await {
                if let Some(id) = user["id"].as_i64() {
                    let id_str = id.to_string();
                    if let Ok(conn) = rusqlite::Connection::open("atlas.db") {
                        let mut contribution_days: Vec<ContributionDay> = Vec::new();
                        if let Ok(mut cstmt) = conn.prepare(
                            "SELECT day, SUM(count) as total
                             FROM contributions
                             WHERE github_id = ?1
                               AND day >= date('now', '-365 days')
                             GROUP BY day
                             ORDER BY day ASC"
                        ) {
                            if let Ok(rows) = cstmt.query_map([&id_str], |row| {
                                Ok(ContributionDay {
                                    date: row.get(0)?,
                                    count: row.get(1)?,
                                })
                            }) {
                                for row in rows.flatten() {
                                    contribution_days.push(row);
                                }
                            }
                        }

                        if let Ok(mut stmt) = conn.prepare("SELECT username, avatar_url, commits, reviews, trust_rating FROM users WHERE github_id = ?1") {
                            let profile = stmt.query_row([&id_str], |row| {
                                Ok(UserProfile {
                                    github_id: id_str.clone(),
                                    username: row.get(0)?,
                                    avatar_url: row.get(1)?,
                                    commits: row.get(2)?,
                                    reviews: row.get(3)?,
                                    trust_rating: row.get(4)?,
                                    contribution_days,
                                })
                            });
                            if let Ok(p) = profile {
                                return Json(p).into_response();
                            }
                        }
                    }
                }
            }
        }
    }
    (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

#[cfg(test)]
mod tests {
    use super::github_authorize_url;

    #[test]
    fn authorize_url_requests_repo_scope() {
        let url = github_authorize_url("abc123", "http://127.0.0.1:3000/api/auth/callback");
        assert!(url.contains("scope=repo"));
    }

    #[test]
    fn authorize_url_contains_client_id_and_redirect_uri() {
        let client_id = "client_xyz";
        let redirect = "http://127.0.0.1:3000/api/auth/callback";
        let url = github_authorize_url(client_id, redirect);

        assert!(url.contains(&format!("client_id={}", client_id)));
        assert!(url.contains(&format!("redirect_uri={}", redirect)));
    }
}
