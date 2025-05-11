use crate::cli::{Cli, PublicKeyValue, SecretKeyValue, get_version};
use std::net::SocketAddr;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tower_http::timeout::TimeoutLayer;

use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::limit::RequestBodyLimitLayer;
use crate::utils::crypto::decode_payload;

#[derive(Debug, Deserialize)]
struct InitRequest {
    config: String,
}

#[derive(Clone)]
struct AppState {
    server_private_key: SecretKeyValue,
    manager_public_key: PublicKeyValue,
    config: Option<String>,
}

type SharedState = Arc<Mutex<AppState>>;

pub async fn pre_run(cli: Cli) -> anyhow::Result<Option<String>> {
    sodiumoxide::init().unwrap();

    // We create a shared shutdown signal to notify the server when it's time to shut down
    let shutdown_signal = Arc::new(Notify::new());
    let shutdown_signal_clone = shutdown_signal.clone();

    let state = Arc::new(Mutex::new(AppState {
        server_private_key: cli.server_private_key,
        manager_public_key: cli.manager_public_key,
        config: None,
    }));

    let state_clone = state.clone();

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2) // 1 request per second
            .burst_size(5) // allow bursts of up to 5
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let init_handler = |State(state): State<SharedState>, Json(payload): Json<InitRequest>| async move {
        let mut data = state.lock().await;

        match decode_payload(
            &payload.config,
            &data.server_private_key,
            &data.manager_public_key,
        ) {
            Ok(decoded_config) => {
                data.config = Some(decoded_config);

                // We have what we need, so shutdown the server
                shutdown_signal.notify_one();

                "Service unsealed and started.".into_response()
            }
            Err(_) => (
                axum::http::StatusCode::BAD_REQUEST,
                "Error decoding payload",
            )
                .into_response(),
        }
    };

    let server = tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(|| async { format!("Ok {}", get_version()) }))
            .route("/init", post(init_handler))
            .with_state(state_clone)
            .layer(RequestBodyLimitLayer::new(1_048_576)) // 1 MB body limit
            .layer((TimeoutLayer::new(Duration::from_secs(5)),))
            .layer(GovernorLayer {
                config: governor_config,
            })
            .into_make_service_with_connect_info::<SocketAddr>(); // Fallback for rate limiter

        let listener = tokio::net::TcpListener::bind(cli.bind).await.unwrap();

        println!("Unsealer listening on {}", cli.bind);

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_signal_clone.notified().await;
            })
            .await
            .unwrap();
    });

    // Wait for the server to shut down
    server.await?;

    Ok(state.lock().await.config.clone())
}

#[cfg(test)]
mod tests;
