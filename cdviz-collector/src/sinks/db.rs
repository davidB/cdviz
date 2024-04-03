use crate::{errors::Result, Message};

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::Instrument;

use super::Sink;

/// The database client config
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    /// The database url (with username, password and the database)
    url: String,

    /// The minimum number of connections to the database to maintain at all times.
    /// minimum > 0, require to have access to the database at startup time,
    /// consume a little more resource on idle
    /// and could increase performance on low load (keep prepared statement,...)
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.min_connections
    pool_connections_min: u32,

    /// The maximum number of connections to the database to open / to maintain.
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.max_connections
    pool_connections_max: u32,
}

/// Build database connections pool
///
/// # Errors
///
/// Fail if we cannot connect to the database
impl TryFrom<Config> for DbSink {
    type Error = crate::errors::Error;

    fn try_from(config: Config) -> Result<Self> {
        let pool_options = PgPoolOptions::new()
            .min_connections(config.pool_connections_min)
            .max_connections(config.pool_connections_max);
        tracing::info!(
            max_connections = pool_options.get_max_connections(),
            min_connections = pool_options.get_min_connections(),
            acquire_timeout = ?pool_options.get_acquire_timeout(),
            idle_timeout = ?pool_options.get_idle_timeout(),
            max_lifetime = ?pool_options.get_max_lifetime(),
            test_before_acquire = pool_options.get_test_before_acquire(),
            "Using the database"
        );

        let pool = pool_options.connect_lazy(&config.url)?;

        Ok(Self { pool })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DbSink {
    pool: PgPool,
}

impl Sink for DbSink {
    async fn send(&self, message: &Message) -> Result<()> {
        store_event(
            &self.pool,
            // TODO build Event from raw json
            Event {
                timestamp: *message.cdevent.timestamp(),
                payload: serde_json::to_value(&message.cdevent)?,
                subject: message.cdevent.subject().content().subject().to_lowercase(),
                predicate: message.cdevent.subject().content().predicate().to_string(),
                version: None,
            },
        )
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct Event {
    timestamp: OffsetDateTime,
    payload: serde_json::Value,
    subject: String,
    predicate: String,
    version: Option<[i32; 3]>,
}

// basic handmade span far to be compliant with
//[opentelemetry-specification/.../database.md](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.22.0/specification/trace/semantic_conventions/database.md)
#[allow(dead_code)]
fn build_otel_span(db_operation: &str) -> tracing::Span {
    tracing::trace_span!(
        target: tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET,
        "DB request",
        db.system = "postgresql",
        // db.statement = stmt,
        db.operation = db_operation,
        otel.name = db_operation, // should be <db.operation> <db.name>.<db.sql.table>,
        otel.kind = "CLIENT",
        otel.status_code = tracing::field::Empty,
    )
}

// store event as json in db (postgresql using sqlx)
async fn store_event(pg_pool: &PgPool, event: Event) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO cdevents_lake (timestamp, payload, subject, predicate, version)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        event.timestamp,
        event.payload,
        event.subject,
        event.predicate,
        event.version.as_ref().map(|x| x.as_slice()),
    )
    .execute(pg_pool)
    .instrument(build_otel_span("INSERT INTO events"))
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;
    use rstest::*;
    use rustainers::images::Postgres;
    use rustainers::runner::{RunOption, Runner};
    use rustainers::Container;

    struct TestContext {
        pub sink: DbSink,
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
    async fn async_pg() -> (DbSink, Container<Postgres>) {
        let runner = Runner::auto().expect("container runner");
        let image = Postgres::default().with_tag("16");
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

        let config = Config {
            url: pg_container
                .url()
                .await
                .expect("find db url")
                .replace("localhost", "127.0.0.1"), // replace localhost by 127.0.0.1 because localhost in ipv6 doesn't work
            pool_connections_min: 1,
            pool_connections_max: 30,
        };
        let dbsink = DbSink::try_from(config).unwrap();
        //Basic initialize the db schema
        //TODO improve the loading, initialisation of the db
        for sql in read_to_string("../cdviz-db/src/schema.sql")
            .unwrap()
            .split(';')
        {
            sqlx::QueryBuilder::new(sql)
                .build()
                .execute(&dbsink.pool)
                .await
                .unwrap();
        }
        // container should be keep, else it is remove on drop
        (dbsink, pg_container)
    }

    // servers() is called once per test, so db could only started several times.
    // We could not used `static` (or the once on fixtures) because statis are not dropped at end of the test
    #[fixture]
    async fn testcontext(#[future] async_pg: (DbSink, Container<Postgres>)) -> TestContext {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::WARN)
            .finish();
        let _tracing_guard = tracing::subscriber::set_default(subscriber);

        let (sink, _db_guard) = async_pg.await;
        TestContext {
            sink,
            _db_guard,
            _tracing_guard,
        }
    }

    #[rstest()]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_send_random_cdevents(#[future] testcontext: TestContext) {
        use proptest::prelude::*;
        use proptest::test_runner::TestRunner;
        use sqlx::Row;
        let testcontext = testcontext.await; // to keep guard & DB up
        let sink = testcontext.sink;
        let mut runner = TestRunner::default();
        let mut count: i64 = sqlx::QueryBuilder::new("SELECT count(*) from cdevents_lake")
            .build()
            .fetch_one(&sink.pool)
            .await
            .unwrap()
            .get(0);

        for _ in 0..1 {
            let val = any::<Message>().new_tree(&mut runner).unwrap();
            sink.send(&val.current()).await.unwrap();
            //TODO check insertion content
            let count_n: i64 = sqlx::QueryBuilder::new("SELECT count(*) from cdevents_lake")
                .build()
                .fetch_one(&sink.pool)
                .await
                .unwrap()
                .get(0);
            count += 1;
            assert_eq!(count_n, count);
        }
    }
}
