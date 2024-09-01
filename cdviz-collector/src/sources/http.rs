use super::{EventSourcePipe, Extractor};
use crate::{
    errors::{self, Error},
    sources::EventSource,
};
use axum::{
    extract::State,
    http,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use errors::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
};

/// The http server config
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    /// Listening host of http server
    /// #[clap(long, env("HTTP_HOST"), default_value = "0.0.0.0")]
    pub(crate) host: IpAddr,

    /// Listening port of http server
    /// #[clap(long, env("HTTP_PORT"), default_value = "8080")]
    pub(crate) port: u16,
}

pub(crate) struct HttpExtractor {
    config: Config,
    next: Arc<Mutex<EventSourcePipe>>,
}

impl HttpExtractor {
    pub fn try_from(value: &Config, next: EventSourcePipe) -> Result<Self> {
        Ok(HttpExtractor { config: value.clone(), next: Arc::new(Mutex::new(next)) })
    }
}
#[derive(Clone)]
struct AppState {
    next: Arc<Mutex<EventSourcePipe>>,
}

impl Extractor for HttpExtractor {
    async fn run(&mut self) -> Result<()> {
        let app_state = AppState { next: Arc::clone(&self.next) };

        let app = app().with_state(app_state);
        // run it
        let addr = &SocketAddr::new(self.config.host, self.config.port);
        tracing::warn!("listening on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service())
            //FIXME gracefull shutdown is in wip for axum 0.7
            // see [axum/examples/graceful-shutdown/src/main.rs at main · tokio-rs/axum](https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs)
            // .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

//TODO make route per extractor/sources
fn app() -> Router<AppState> {
    // build our application with a route
    Router::new()
        .route("/cdevents", post(events_collect))
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

//TODO support events in cloudevents format (extract info from headers)
//TODO try [deser](https://crates.io/crates/deserr) to return good error
//TODO use cloudevents
//TODO add metadata & headers info into SourceEvent
//TODO log & convert error
#[tracing::instrument(skip(app_state, body))]
async fn events_collect(
    State(app_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<http::StatusCode> {
    tracing::trace!("received {:?}", &body);
    let event = EventSource { metadata: serde_json::Value::Null, header: HashMap::new(), body };
    let mut next = app_state.next.lock().unwrap();
    next.as_mut().send(event)?;
    Ok(http::StatusCode::CREATED)
}

impl IntoResponse for Error {
    //TODO report the trace_id into the message to help to debug
    fn into_response(self) -> axum::response::Response {
        // let (status, error_message) = match self {
        //     Error::Db(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        //     _ => (http::StatusCode::INTERNAL_SERVER_ERROR, "".to_string()),
        // };
        let (status, error_message) = (http::StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
        tracing::warn!(?error_message);
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use axum_test::{TestServer, TestServerConfig};
//     use rstest::*;
//     use rustainers::images::Postgres;
//     use rustainers::runner::{RunOption, Runner};
//     use rustainers::Container;
//     use serde_json::json;

//     struct TestContext {
//         http: TestServer,
//         // keep db container to drop it after the test
//         _db_guard: Container<Postgres>,
//         // keep tracing subscriber
//         _tracing_guard: tracing::subscriber::DefaultGuard,
//     }

//     // #[fixture]
//     // //#[once] // only work with non-async, non generic fixtures
//     // // workaround explained at [Async once fixtures · Issue #141 · la10736/rstest](https://github.com/la10736/rstest/issues/141)
//     // // no drop call on the fixture like on static
//     // fn pg() -> (PgPool, Container<Postgres>) {
//     //     futures::executor::block_on(async { async_pg().await })
//     // }

//     #[fixture]
//     async fn async_pg() -> (PgPool, Container<Postgres>) {
//         let runner = Runner::auto().expect("container runner");
//         let image = Postgres::default().with_tag("16.1");
//         // runner should remove the container after the test (on drop)
//         let pg_container = runner
//             .start_with_options(
//                 image,
//                 RunOption::builder()
//                     .with_remove(true)
//                     //.with_name("test_cdviz") // use random name for parallel test
//                     .build(),
//             )
//             .await
//             .expect("start container");
//         let db_url = pg_container
//             .url()
//             .await
//             .expect("find db url")
//             .replace("localhost", "127.0.0.1") // replace localhost by 127.0.0.1 because localhost in ipv6 doesn't work
//             ;
//         let pg = PgPool::connect(&db_url).await.expect("build a pg pool");
//         // run migrations
//         sqlx::migrate!("../migrations")
//             .run(&pg)
//             .await
//             .expect("migrate db");
//         // container should be keep, else it is remove on drop
//         (pg, pg_container)
//     }

//     // servers() is called once per test, so db could only started several times.
//     // We could not used `static` (or the once on fixtures) because statis are not dropped at end of the test
//     #[fixture]
//     async fn testcontext(#[future] async_pg: (PgPool, Container<Postgres>)) -> TestContext {
//         let subscriber = tracing_subscriber::FmtSubscriber::builder()
//             .with_max_level(tracing::Level::WARN)
//             .finish();
//         let _tracing_guard = tracing::subscriber::set_default(subscriber);

//         let (pg_pool, db) = async_pg.await;
//         let app_state = AppState {
//             pg_pool: Arc::new(pg_pool.clone()),
//         };
//         let app = app().with_state(app_state);

//         let config = TestServerConfig::builder()
//             // Preserve cookies across requests
//             // for the session cookie to work.
//             .save_cookies()
//             .expect_success_by_default()
//             .mock_transport()
//             .build();

//         TestContext {
//             http: TestServer::new_with_config(app, config).unwrap(),
//             _db_guard: db,
//             _tracing_guard,
//         }
//     }

//     #[rstest]
//     #[tokio::test(flavor = "multi_thread")]
//     // test health endpoint
//     async fn test_readyz(#[future] testcontext: TestContext) {
//         let resp = testcontext.await.http.get("/readyz").await;
//         resp.assert_status_ok();
//     }

//     #[rstest]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_post_cdevents(#[future] testcontext: TestContext) {
//         let resp = testcontext
//             .await
//             .http
//             .post("/cdevents")
//             .json(&json!({
//                 "bar": "foo",
//             }))
//             .await;
//         resp.assert_text("");
//         resp.assert_status(http::StatusCode::CREATED);
//     }
// }
