use axum::{
    http,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Router,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    //TODO use logfmt format (with traceid,...) see [tracing-logfmt-otel](https://github.com/elkowar/tracing-logfmt-otel)
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    let app = app();
    // run it
    let addr = &"0.0.0.0:3000".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        //FIXME gracefull shutdown is in wip for axum 0.7
        // see [axum/examples/graceful-shutdown/src/main.rs at main · tokio-rs/axum](https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs)
        // .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
        .route("/cdevents", post(cdevents_collect))
        // include trace context as header into the response
        .layer(OtelInResponseLayer)
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/healthz", get(health)) // request processed without span / trace
        .route("/readyz", get(health)) // request processed without span / trace
}

async fn health() -> impl IntoResponse {
    http::StatusCode::OK
}

#[tracing::instrument]
//TODO validate format of cdevents JSON
//TODO support events in cloudevents format (extract info from headers)
//TODO try [deser](https://crates.io/crates/deserr) to return good error
async fn cdevents_collect(event: String) -> impl IntoResponse {
    // TODO store json into DB
    tracing::debug!("received cloudevent {}", &event);
    http::StatusCode::CREATED
}
