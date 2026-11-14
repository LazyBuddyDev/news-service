//! Optional local HTTP adapter for clients outside the desktop host.
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use lazybuddy_news::{feeds, protocol};
use serde_json::{Value, json};
use std::sync::Arc;

struct Service {
    token: String,
    client: ureq::Agent,
    permit: Arc<tokio::sync::Semaphore>,
    state: std::sync::Mutex<lazybuddy_news::state::Service>,
}

async fn rpc(
    State(service): State<Arc<Service>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    if headers.contains_key("origin")
        || headers.get("authorization").and_then(|v| v.to_str().ok())
            != Some(service.token.as_str())
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let permit = service
        .permit
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)?;
    let reply = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        service
            .state
            .lock()
            .expect("news service state lock")
            .handle(&service.client, &body)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(reply))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token =
        std::env::var("LAZYBUDDY_API_TOKEN").map_err(|_| "LAZYBUDDY_API_TOKEN is required")?;
    if token.len() < 32
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("API token must contain at least 32 ASCII letters/digits/-/_".into());
    }
    let port: u16 = std::env::var("LAZYBUDDY_API_PORT")
        .unwrap_or_else(|_| "0".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    let address = listener.local_addr()?;
    let service = Arc::new(Service {
        token: format!("Bearer {token}"),
        client: feeds::client(),
        permit: Arc::new(tokio::sync::Semaphore::new(1)),
        state: std::sync::Mutex::new(lazybuddy_news::state::Service::new(
            lazybuddy_news::state::Service::default_path(),
        )),
    });
    let app = Router::new()
        .route("/v1/news/rpc", post(rpc))
        .layer(DefaultBodyLimit::max(protocol::MAX_REQUEST_BYTES))
        .with_state(service);
    println!(
        "{}",
        json!({"api_version":1,"address":format!("http://{address}/v1/news/rpc")})
    );
    axum::serve(listener, app).await?;
    Ok(())
}
