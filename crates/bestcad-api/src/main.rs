//! BestCAD API server.
//!
//! Axum-based HTTP server providing geometry endpoints for the browser UI.
//! In production, also serves the built web assets from a static directory.

use axum::{
    Router,
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use bestcad_types::{BoxParams, ErrorResponse, GeometryResponse, HealthResponse};
use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bestcad_api=info,tower_http=info".into()),
        )
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    // Build API routes
    let api_routes = Router::new()
        .route("/health", get(health))
        .route("/geometry/box", post(create_box))
        .route("/geometry/export-step", post(export_step));

    let mut app = Router::new()
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Serve static web assets in production
    if let Ok(static_dir) = env::var("BESTCAD_STATIC_DIR") {
        let static_path = PathBuf::from(&static_dir);
        if static_path.exists() {
            info!("Serving static assets from {}", static_dir);
            // Serve the SPA: any non-API, non-file request falls back to index.html
            app = app
                .fallback_service(
                    ServeDir::new(&static_path)
                        .fallback(
                            tower_http::services::ServeFile::new(
                                static_path.join("index.html"),
                            ),
                        ),
                );
        } else {
            tracing::warn!("BESTCAD_STATIC_DIR={} does not exist", static_dir);
        }
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("BestCAD API starting on http://{}", addr);
    info!("OCCT version: {}", bestcad_geometry::version());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Health check endpoint.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        occt_version: bestcad_geometry::version(),
    })
}

/// Create a tessellated box.
async fn create_box(
    Json(params): Json<BoxParams>,
) -> Result<Json<GeometryResponse>, AppError> {
    info!(
        "Creating box: {}×{}×{}",
        params.width, params.height, params.depth
    );

    let response = tokio::task::spawn_blocking(move || {
        bestcad_geometry::create_box(params.width, params.height, params.depth)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
    .map_err(|e| AppError::Geometry(e.to_string()))?;

    info!(
        "Box created: {} faces, {} triangles, volume={:.1}",
        response.face_count,
        response.mesh.indices.len() / 3,
        response.volume
    );

    Ok(Json(response))
}

/// Export a box as STEP file.
async fn export_step(
    Json(params): Json<BoxParams>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Exporting STEP: {}×{}×{}",
        params.width, params.height, params.depth
    );

    let step_bytes = tokio::task::spawn_blocking(move || {
        bestcad_geometry::export_step(params.width, params.height, params.depth)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
    .map_err(|e| AppError::Geometry(e.to_string()))?;

    info!("STEP exported: {} bytes", step_bytes.len());

    Ok((
        StatusCode::OK,
        [
            ("content-type", "application/octet-stream"),
            (
                "content-disposition",
                "attachment; filename=\"bestcad-export.step\"",
            ),
        ],
        step_bytes,
    ))
}

/// Application error type that converts to HTTP responses.
enum AppError {
    Geometry(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Geometry(msg) => {
                error!("Geometry error: {msg}");
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::Internal(msg) => {
                error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: message,
                detail: None,
            }),
        )
            .into_response()
    }
}
