use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use izar_core::{
    network::IzarNetwork,
    types::transaction::{IzarTransaction, Priority},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::Level;

use super::{store::RelayerStore, IzarRelayer};

impl<I: IzarNetwork> IzarRelayer<I> {
    pub async fn serve(self_: Arc<IzarRelayer<I>>) -> anyhow::Result<()> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
            .allow_headers([axum::http::header::CONTENT_TYPE]);

        let store = self_.store().clone();
        let router = Router::new()
            .route("/exec", post(execute))
            .route("/speedup", post(speedup))
            .with_state(store)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            );

        let addr = SocketAddr::from(([0, 0, 0, 0], self_.port));
        tracing::info!("rest server listening on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router.into_make_service()).await?;
        Ok(())
    }
}

async fn execute(State(store): State<RelayerStore>, Json(tx): Json<IzarTransaction>) -> impl IntoResponse {
    // check if already finalized
    match store.finalize().get(&tx.from_chain_tx_hash) {
        Ok(Some(tx)) => {
            tracing::info!("tx already finalized: {:?}", tx);
            (StatusCode::OK, tx.to_chain_tx_hash.unwrap()).into_response()
        }
        Err(e) => {
            tracing::error!("failed to get tx from db: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
        Ok(None) => {
            if let Err(e) = store.execute().insert(tx.order_key(), tx.clone()) {
                tracing::error!("failed to insert tx to db: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            (StatusCode::ACCEPTED, format!("already added to queue: {tx:?}")).into_response()
        }
    }
}

async fn speedup(State(store): State<RelayerStore>, Json(mut tx): Json<IzarTransaction>) -> impl IntoResponse {
    tx.priority = Priority::High;
    // check if already finalized
    match store.finalize().get(&tx.from_chain_tx_hash) {
        Ok(Some(tx)) => {
            tracing::info!("tx already finalized: {:?}", tx);
            (StatusCode::OK, tx.to_chain_tx_hash.unwrap()).into_response()
        }
        Err(e) => {
            tracing::error!("failed to get tx from db: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
        Ok(None) => {
            if let Err(e) = store.execute().insert(tx.order_key(), tx.clone()) {
                tracing::error!("failed to insert tx to db: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            (StatusCode::ACCEPTED, format!("already added to queue: {tx:?}")).into_response()
        }
    }
}
