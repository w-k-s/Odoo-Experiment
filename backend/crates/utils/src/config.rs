/// Connection settings for the Odoo JSON-RPC endpoint.
#[derive(Debug, Clone)]
pub struct OdooConfig {
    pub url: String,
    pub db: String,
    pub login: String,
    /// API key minted for the user (used in place of a password).
    pub api_key: String,
}

/// Machine-to-machine credentials for the Auth0 Management API, used by the
/// identity integration to resolve a caller's profile by `sub`.
#[derive(Debug, Clone)]
pub struct Auth0MgmtConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub group_id: String,
    pub bootstrap_servers: String,
    pub topics: Vec<String>,
}

/// Runtime configuration, read from environment variables.
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub pool_max_size: usize,
    /// Name of the loyalty program to bootstrap on startup if none exists.
    pub bootstrap_program_name: String,
    /// Auth0 tenant domain, e.g. `wks-bakery.eu.auth0.com`.
    pub auth0_domain: String,
    /// Auth0 audience the tokens are issued for (the SPA client id for ID-token auth).
    pub auth0_audience: String,
    /// Auth0 Management API M2M credentials (identity integration).
    pub auth0_mgmt: Auth0MgmtConfig,
    /// Odoo JSON-RPC connection settings.
    pub odoo: OdooConfig,
    pub consumer: ConsumerConfig,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let bind_addr =
            std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
        let pool_max_size = std::env::var("POOL_MAX_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8);
        let bootstrap_program_name = std::env::var("BOOTSTRAP_PROGRAM_NAME")
            .unwrap_or_else(|_| "UAE Bakery Rewards".to_string());

        let auth0_domain = std::env::var("AUTH0_DOMAIN").expect("AUTH0_DOMAIN must be set");
        let auth0_audience =
            std::env::var("AUTH0_AUDIENCE").expect("AUTH0_AUDIENCE must be set");

        let auth0_mgmt = Auth0MgmtConfig {
            client_id: std::env::var("AUTH0_MGMT_CLIENT_ID")
                .expect("AUTH0_MGMT_CLIENT_ID must be set"),
            client_secret: std::env::var("AUTH0_MGMT_CLIENT_SECRET")
                .expect("AUTH0_MGMT_CLIENT_SECRET must be set"),
        };

        let odoo = OdooConfig {
            url: std::env::var("ODOO_URL").unwrap_or_else(|_| "http://odoo:8069".to_string()),
            db: std::env::var("ODOO_DB").unwrap_or_else(|_| "odoo".to_string()),
            login: std::env::var("ODOO_LOGIN").unwrap_or_else(|_| "loyalty-bot".to_string()),
            api_key: std::env::var("ODOO_API_KEY").unwrap_or_default(),
        };

        let consumer = ConsumerConfig { 
            group_id: std::env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "1".to_string()), 
            bootstrap_servers: std::env::var("KAFKA_BOOTSTRAP_SERVERS").expect("KAFKA_BOOTSTRAP_SERVERS must be set"), 
            topics: std::env::var("KAFKA_TOPICS").expect("KAFKA_TOPICS must be set")
                .split(',')
                .map(String::from)
                .collect() };

        Self {
            database_url,
            bind_addr,
            pool_max_size,
            bootstrap_program_name,
            auth0_domain,
            auth0_audience,
            auth0_mgmt,
            odoo,
            consumer
        }
    }
}
