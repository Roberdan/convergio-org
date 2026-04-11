//! HTTP API routes for convergio-org.

use axum::Router;

/// Returns the router for this crate's API endpoints.
pub fn routes() -> Router {
    Router::new()
    // .route("/api/org/health", get(health))
}
