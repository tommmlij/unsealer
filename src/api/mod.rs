use crate::cli::{Cli, get_version};

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use serde::Deserialize;
use sodiumoxide::crypto::box_;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tower_http::timeout::TimeoutLayer;

#[derive(Debug, Deserialize)]
struct InitRequest {
    config: String,
}

#[derive(Clone)]
struct AppState {
    cli: Cli,
    shutdown_trigger: Arc<Notify>,
    config: Option<String>,
}

type SharedState = Arc<Mutex<AppState>>;

async fn init_handler(
    State(state): State<SharedState>,
    Json(payload): Json<InitRequest>,
) -> String {
    
    let decoded = URL_SAFE.decode(payload.config).unwrap();

    if decoded.len() < box_::NONCEBYTES {
        return "Invalid encrypted payload".into();
    }

    let nonce = box_::Nonce::from_slice(&decoded[..box_::NONCEBYTES]).unwrap();
    let ciphertext = &decoded[box_::NONCEBYTES..];

    let mut data = state.lock().await;

    let decrypted = match box_::open(
        ciphertext,
        &nonce,
        &data.cli.manager_public_key,
        &data.cli.server_private_key,
    ) {
        Ok(data) => data,
        Err(_) => return "Decryption failed".into(),
    };

    data.config = Some(String::from_utf8(decrypted).unwrap());

    data.shutdown_trigger.clone().notify_one();

    "Service unsealed and started.".into()
}

pub async fn pre_run(cli: Cli) -> anyhow::Result<Option<String>> {
    sodiumoxide::init().unwrap();

    let shutdown_signal = Arc::new(Notify::new());
    let shutdown_signal_clone = shutdown_signal.clone();

    let state = Arc::new(Mutex::new(AppState {
        cli: cli.clone(),
        shutdown_trigger: shutdown_signal_clone,
        config: None
    }));

    let state_clone = state.clone();

    let server = tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(|| async { format!("Ok {}", get_version()) }))
            .route("/init", post(init_handler))
            .with_state(state_clone)
            .layer((TimeoutLayer::new(Duration::from_secs(5)),));

        let listener = tokio::net::TcpListener::bind(cli.bind).await.unwrap();

        println!("Unsealer listening on {}", cli.bind);

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_signal.notified().await;
            })
            .await
            .unwrap();
    });

    // Wait for the server to shut down
    server.await?;

    let result = {
        let data = state.lock().await;
        data.config.clone()
    };

    Ok(result)
}
