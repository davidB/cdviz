use std::net::IpAddr;

#[derive(Debug, Clone, clap::Parser)]
pub struct Settings {
    /// The http server settings
    #[command(flatten)]
    pub http_settings: HttpSettings,

    /// The database client settings
    #[command(flatten)]
    pub db_settings: DbSettings,
}

/// The http server settings
#[derive(Debug, Clone, clap::Parser)]
pub struct HttpSettings {
    /// Listening host of http server
    #[clap(long, env("HTTP_HOST"), default_value = "0.0.0.0")]
    pub host: IpAddr,

    /// Listening port of http server
    #[clap(long, env("HTTP_PORT"), default_value = "8080")]
    pub port: u16,
}

/// The database client settings
#[derive(Debug, Clone, clap::Parser)]
pub struct DbSettings {
    /// The database url (with username, password and the database)
    #[clap(
        long = "database",
        env("DATABASE_URL"),
        default_value = "postgresql://postgres:passwd@localhost:5432/cdviz"
    )]
    pub url: String,

    /// The minimum number of connections to the database to maintain at all times.
    /// minimum > 0, require to have access to the database at startup time,
    /// consume a little more resource on idle
    /// and could increase performance on low load (keep prepared statement,...)
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.min_connections
    #[clap(
        long = "database-pool-connections-min",
        env("DATABASE_POOL_CONNECTIONS_MIN"),
        default_value = "1"
    )]
    pub pool_connections_min: u32,

    /// The maximum number of connections to the database to open / to maintain.
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.max_connections
    #[clap(
        long = "database-pool-connections-max",
        env("DATABASE_POOL_CONNECTIONS_MAX"),
        default_value = "10"
    )]
    pub pool_connections_max: u32,
}
