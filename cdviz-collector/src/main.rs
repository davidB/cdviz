mod db;
mod errors;
mod settings;

use axum::{
    extract::State,
    http,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Json, Router,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use clap::Parser;
use errors::Result;
use serde_json::json;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use time::OffsetDateTime;

#[derive(Clone)]
struct AppState {
    pg_pool: Arc<PgPool>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    //TODO use logfmt format (with traceid,...) see [tracing-logfmt-otel](https://github.com/elkowar/tracing-logfmt-otel)
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    let settings = settings::Settings::parse();

    let app = app().with_state(app_state(&settings).await?);
    // run it
    let addr = &SocketAddr::new(settings.http_settings.host, settings.http_settings.port);
    tracing::warn!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        //FIXME gracefull shutdown is in wip for axum 0.7
        // see [axum/examples/graceful-shutdown/src/main.rs at main · tokio-rs/axum](https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs)
        // .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router<AppState> {
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

async fn app_state(settings: &settings::Settings) -> Result<AppState> {
    let pg_pool = db::build_pool(&settings.db_settings).await?;
    Ok(AppState {
        pg_pool: Arc::new(pg_pool),
    })
}

async fn health() -> impl IntoResponse {
    http::StatusCode::OK
}

//TODO validate format of cdevents JSON
//TODO support events in cloudevents format (extract info from headers)
//TODO try [deser](https://crates.io/crates/deserr) to return good error
#[tracing::instrument(skip(app_state, payload))]
async fn cdevents_collect(
    State(app_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<http::StatusCode> {
    // TODO store json into DB
    tracing::debug!("received cloudevent {}", &payload);
    db::store_event(
        &app_state.pg_pool,
        // TODO build Event from raw json
        Event {
            timestamp: OffsetDateTime::now_utc(),
            raw: payload,
        },
    )
    .await?;
    Ok(http::StatusCode::CREATED)
}

impl IntoResponse for errors::Error {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            errors::Error::Db(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        tracing::warn!(?error_message);
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(sqlx::FromRow)]
struct Event {
    timestamp: OffsetDateTime,
    raw: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::{TestServer, TestServerConfig};
    use rstest::*;
    use rustainers::images::Postgres;
    use rustainers::runner::{RunOption, Runner};
    use rustainers::Container;
    use serde_json::json;

    struct TestContext {
        http: TestServer,
        // keep db container to drop it after the test
        _db_guard: Container<Postgres>,
        // keep tracing subscriber
        _tracing_guard: tracing::subscriber::DefaultGuard,
    }

    // #[fixture]
    // //#[once] // only work with non-async, non generic fixtures
    // // workaround explained at [Async once fixtures · Issue #141 · la10736/rstest](https://github.com/la10736/rstest/issues/141)
    // // no drop call on the fixture like on static
    // fn pg() -> (PgPool, Container<Postgres>) {
    //     futures::executor::block_on(async { async_pg().await })
    // }

    #[fixture]
    async fn async_pg() -> (PgPool, Container<Postgres>) {
        let runner = Runner::auto().expect("container runner");
        let image = Postgres::default().with_tag("16.1");
        // runner should remove the container after the test (on drop)
        let pg_container = runner
            .start_with_options(
                image,
                RunOption::builder()
                    .with_remove(true)
                    //.with_name("test_cdviz") // use random name for parallel test
                    .build(),
            )
            .await
            .expect("start container");
        let db_url = pg_container
            .url()
            .await
            .expect("find db url")
            .replace("localhost", "127.0.0.1") // replace localhost by 127.0.0.1 because localhost in ipv6 doesn't work
            ;
        let pg = PgPool::connect(&db_url).await.expect("build a pg pool");
        // run migrations
        sqlx::migrate!("../migrations")
            .run(&pg)
            .await
            .expect("migrate db");
        // container should be keep, else it is remove on drop
        (pg, pg_container)
    }

    // servers() is called once per test, so db could only started several times.
    // We could not used `static` (or the once on fixtures) because statis are not dropped at end of the test
    #[fixture]
    async fn testcontext(#[future] async_pg: (PgPool, Container<Postgres>)) -> TestContext {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::WARN)
            .finish();
        let _tracing_guard = tracing::subscriber::set_default(subscriber);

        let (pg_pool, db) = async_pg.await;
        let app_state = AppState {
            pg_pool: Arc::new(pg_pool.clone()),
        };
        let app = app().with_state(app_state);

        let config = TestServerConfig::builder()
            // Preserve cookies across requests
            // for the session cookie to work.
            .save_cookies()
            .expect_success_by_default()
            .mock_transport()
            .build();

        TestContext {
            http: TestServer::new_with_config(app, config).unwrap(),
            _db_guard: db,
            _tracing_guard,
        }
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    // test health endpoint
    async fn test_readyz(#[future] testcontext: TestContext) {
        let resp = testcontext.await.http.get("/readyz").await;
        resp.assert_status_ok();
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_cdevents(#[future] testcontext: TestContext) {
        let resp = testcontext
            .await
            .http
            .post("/cdevents")
            .json(&json!({
                "bar": "foo",
            }))
            .await;
        resp.assert_text("");
        resp.assert_status(http::StatusCode::CREATED);
    }
}
