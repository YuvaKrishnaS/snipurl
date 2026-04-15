use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    routing::{delete, get, post},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

struct AppState {
    db: SqlitePool,
    api_key: String,
    base_url: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Link {
    id: i64,
    code: String,
    original_url: String,
    clicks: i64,
    created_at: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct ClickLog {
    id: i64,
    code: String,
    user_agent: String,
    clicked_at: String,
}

#[derive(Deserialize)]
struct ShortenRequest {
    url: String,
    custom_code: Option<String>,
}

#[derive(Serialize)]
struct ShortenResponse {
    code: String,
    short_url: String,
    original_url: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[derive(Serialize)]
struct GlobalStats {
    total_links: i64,
    total_clicks: i64,
    top_links: Vec<Link>,
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>SnipURL</title>
<style>
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif; background: #0d0d0d; color: #e2e2e2; min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem; }
.card { background: #161616; border: 1px solid #252525; border-radius: 14px; padding: 2.25rem; width: 100%; max-width: 460px; }
h1 { font-size: 1.75rem; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 0.3rem; }
.sub { color: #666; font-size: 0.875rem; margin-bottom: 1.75rem; }
label { display: block; font-size: 0.8rem; font-weight: 500; color: #888; margin-bottom: 0.35rem; margin-top: 0.9rem; }
label:first-of-type { margin-top: 0; }
input { width: 100%; padding: 0.6rem 0.9rem; background: #0d0d0d; border: 1px solid #252525; border-radius: 8px; color: #e2e2e2; font-size: 0.9rem; outline: none; transition: border-color 0.15s; }
input:focus { border-color: #4f98a3; }
button { width: 100%; margin-top: 1.25rem; padding: 0.7rem; background: #4f98a3; border: none; border-radius: 8px; color: #fff; font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: background 0.15s; }
button:hover { background: #3a7f8a; }
.result { margin-top: 1rem; padding: 0.75rem 0.9rem; background: #0d0d0d; border: 1px solid #252525; border-radius: 8px; font-size: 0.875rem; word-break: break-all; display: none; line-height: 1.5; }
.result a { color: #4f98a3; text-decoration: none; }
.result a:hover { text-decoration: underline; }
.stats { display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; margin-top: 1.5rem; }
.stat { background: #0d0d0d; border: 1px solid #252525; border-radius: 8px; padding: 0.9rem 1rem; text-align: center; }
.stat .num { font-size: 1.6rem; font-weight: 700; color: #4f98a3; letter-spacing: -0.02em; }
.stat .lbl { font-size: 0.75rem; color: #555; margin-top: 0.15rem; }
.err { color: #e57373; }
.ok { color: #81c784; }
.divider { border: none; border-top: 1px solid #1f1f1f; margin: 1.5rem 0 0; }
.api-note { font-size: 0.78rem; color: #555; margin-top: 1rem; line-height: 1.5; }
.api-note code { background: #111; padding: 0.1rem 0.35rem; border-radius: 4px; font-size: 0.76rem; color: #888; }
</style>
</head>
<body>
<div class="card">
  <h1>SnipURL</h1>
  <p class="sub">URL shortener with click analytics</p>
  <label>Long URL</label>
  <input id="url" type="url" placeholder="https://example.com/very/long/url" />
  <label>Custom code <span style="color:#3a3a3a">(optional)</span></label>
  <input id="code" type="text" placeholder="e.g. my-link" />
  <label>API key</label>
  <input id="key" type="password" placeholder="required to create links" />
  <button onclick="shorten()">Shorten URL</button>
  <div class="result" id="result"></div>
  <div class="stats">
    <div class="stat">
      <div class="num">TOTAL_LINKS</div>
      <div class="lbl">links created</div>
    </div>
    <div class="stat">
      <div class="num">TOTAL_CLICKS</div>
      <div class="lbl">total clicks</div>
    </div>
  </div>
  <hr class="divider">
  <p class="api-note">API: <code>POST /shorten</code> &middot; <code>GET /links</code> &middot; <code>GET /analytics</code> &middot; <code>GET /stats/:code</code> &middot; <code>DELETE /links/:code</code></p>
</div>
<script>
async function shorten() {
    const url = document.getElementById('url').value.trim();
    const code = document.getElementById('code').value.trim();
    const key = document.getElementById('key').value.trim();
    const res = document.getElementById('result');
    res.style.display = 'block';
    if (!url) { res.innerHTML = '<span class="err">URL is required</span>'; return; }
    const body = { url };
    if (code) body.custom_code = code;
    try {
        const r = await fetch('/shorten', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'x-api-key': key },
            body: JSON.stringify(body)
        });
        const data = await r.json();
        if (r.ok) {
            res.innerHTML = '<span class="ok">Created</span> &rarr; <a href="' + data.short_url + '" target="_blank">' + data.short_url + '</a>';
        } else {
            res.innerHTML = '<span class="err">' + data.message + '</span>';
        }
    } catch(e) {
        res.innerHTML = '<span class="err">Request failed</span>';
    }
}
document.getElementById('url').addEventListener('keydown', function(e) {
    if (e.key === 'Enter') shorten();
});
</script>
</body>
</html>"#;

fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), (StatusCode, Json<ApiResponse>)> {
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided != api_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                message: "invalid or missing API key".to_string(),
            }),
        ));
    }
    Ok(())
}

fn generate_code() -> String {
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..6).map(|_| chars[rng.random_range(0..chars.len())]).collect()
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let total_links: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM links")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let total_clicks: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(clicks), 0) FROM links")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    Html(
        INDEX_HTML
            .replace("TOTAL_LINKS", &total_links.0.to_string())
            .replace("TOTAL_CLICKS", &total_clicks.0.to_string()),
    )
}

async fn health() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "ok".to_string(),
    })
}

async fn shorten_url(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ShortenRequest>,
) -> Result<(StatusCode, Json<ShortenResponse>), (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    if !body.url.starts_with("http://") && !body.url.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                message: "URL must start with http:// or https://".to_string(),
            }),
        ));
    }

    let code = match body.custom_code {
        Some(ref c) if !c.is_empty() => c.clone(),
        _ => generate_code(),
    };

    sqlx::query("INSERT INTO links (code, original_url) VALUES (?, ?)")
        .bind(&code)
        .bind(&body.url)
        .execute(&state.db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                (
                    StatusCode::CONFLICT,
                    Json(ApiResponse {
                        message: format!("code '{}' is already taken, try a different custom_code", code),
                    }),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        message: format!("database error: {}", e),
                    }),
                )
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ShortenResponse {
            short_url: format!("{}/{}", state.base_url, code),
            code: code.clone(),
            original_url: body.url,
        }),
    ))
}

async fn redirect_link(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Json<ApiResponse>)> {
    let row = sqlx::query_as::<_, (String,)>("SELECT original_url FROM links WHERE code = ?")
        .bind(&code)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("database error: {}", e),
                }),
            )
        })?;

    match row {
        Some((original_url,)) => {
            let user_agent = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

            let _ = sqlx::query("INSERT INTO clicks (code, user_agent) VALUES (?, ?)")
                .bind(&code)
                .bind(&user_agent)
                .execute(&state.db)
                .await;

            let _ = sqlx::query("UPDATE links SET clicks = clicks + 1 WHERE code = ?")
                .bind(&code)
                .execute(&state.db)
                .await;

            Ok(Redirect::to(&original_url))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                message: format!("no link found for '{}'", code),
            }),
        )),
    }
}

async fn list_links(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Link>>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let links: Vec<Link> = sqlx::query_as(
        "SELECT id, code, original_url, clicks, created_at FROM links ORDER BY clicks DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                message: format!("database error: {}", e),
            }),
        )
    })?;

    Ok(Json(links))
}

async fn link_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let link = sqlx::query_as::<_, Link>(
        "SELECT id, code, original_url, clicks, created_at FROM links WHERE code = ?",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                message: format!("database error: {}", e),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                message: format!("link '{}' not found", code),
            }),
        )
    })?;

    let recent_clicks: Vec<ClickLog> = sqlx::query_as(
        "SELECT id, code, user_agent, clicked_at FROM clicks WHERE code = ? ORDER BY id DESC LIMIT 20",
    )
    .bind(&code)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                message: format!("database error: {}", e),
            }),
        )
    })?;

    Ok(Json(serde_json::json!({
        "link": link,
        "recent_clicks": recent_clicks,
    })))
}

async fn global_analytics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<GlobalStats>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let total_links: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM links")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("database error: {}", e),
                }),
            )
        })?;

    let total_clicks: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(clicks), 0) FROM links")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("database error: {}", e),
                }),
            )
        })?;

    let top_links: Vec<Link> = sqlx::query_as(
        "SELECT id, code, original_url, clicks, created_at FROM links ORDER BY clicks DESC LIMIT 5",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                message: format!("database error: {}", e),
            }),
        )
    })?;

    Ok(Json(GlobalStats {
        total_links: total_links.0,
        total_clicks: total_clicks.0,
        top_links,
    }))
}

async fn delete_link(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let result = sqlx::query("DELETE FROM links WHERE code = ?")
        .bind(&code)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("database error: {}", e),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                message: format!("link '{}' not found", code),
            }),
        ));
    }

    let _ = sqlx::query("DELETE FROM clicks WHERE code = ?")
        .bind(&code)
        .execute(&state.db)
        .await;

    Ok(Json(ApiResponse {
        message: format!("link '{}' deleted", code),
    }))
}

async fn not_found() -> (StatusCode, Json<ApiResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            message: "not found".to_string(),
        }),
    )
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:snipurl.db?mode=rwc".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let db = SqlitePool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            original_url TEXT NOT NULL,
            clicks INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&db)
    .await
    .expect("failed to create links table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clicks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL,
            user_agent TEXT NOT NULL DEFAULT 'unknown',
            clicked_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&db)
    .await
    .expect("failed to create clicks table");

    let state = Arc::new(AppState { db, api_key, base_url });

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/shorten", post(shorten_url))
        .route("/links", get(list_links))
        .route("/analytics", get(global_analytics))
        .route("/stats/{code}", get(link_stats))
        .route("/links/{code}", delete(delete_link))
        .route("/{code}", get(redirect_link))
        .fallback(not_found)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    println!("SnipURL running on http://{}", addr);
    axum::serve(listener, app).await.expect("server error");
}