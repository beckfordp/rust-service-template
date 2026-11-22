// SPDX-License-Identifier: MIT

//! The HTTP server: routes, binding and graceful shutdown. The health routes,
//! tapir's plain-text errors and the HTTP edge (CORS, the Allow-less 405) are
//! gluon-rs `service-basics`.

use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use service_basics::cors::{CorsPolicy, Unrouted};
use service_basics::edge::edge;
use service_basics::health::{self, Readiness};
use service_basics::text_error::{self, TextError, method_not_allowed};
use tokio::net::TcpListener;

use crate::examples::get_example;
use crate::store::Examples;

/// How long readiness waits for Postgres to answer.
const READY_PING: Duration = Duration::from_secs(2);

/// What the handlers share.
#[derive(Clone)]
pub struct AppState<S> {
    /// The `example` table.
    pub store: S,
}

/// Ready when Postgres answers a ping within 2 s.
impl<S: Examples> Readiness for AppState<S> {
    async fn ready(&self) -> bool {
        self.store.ping(READY_PING).await
    }
}

/// The service: its routes ([`routes`]) behind the Scala services' edge, CORS
/// outermost (`AppRoutes`; set the exposed headers from the service's
/// `Cors.scala`).
pub fn app<S: Examples>(store: S) -> Router {
    edge(routes(store), CorsPolicy::default())
}

/// The routes: health, the example, the docs (`crate::docs`), tapir's 405 for a known path with another
/// method and 404 for any other.
pub fn routes<S: Examples>(store: S) -> Router {
    health::routes()
        .route(
            "/examples/{id}",
            get(get_example::<S>).head(method_not_allowed),
        )
        .method_not_allowed_fallback(method_not_allowed)
        .fallback(not_found)
        .with_state(AppState { store })
        .merge(crate::docs::router())
}

/// 404 for any unknown path, marked [`Unrouted`] for CORS.
async fn not_found() -> (Unrouted, TextError) {
    (Unrouted, text_error::not_found().await)
}

/// Serves the API on `0.0.0.0:<port>` until `shutdown` completes.
///
/// # Errors
/// When the port can't be bound or serving fails.
pub async fn run<S: Examples>(
    port: u16,
    store: S,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))).await?;
    axum::serve(listener, app(store))
        .with_graceful_shutdown(shutdown)
        .await
}
