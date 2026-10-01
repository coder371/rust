pub struct Config {
    pub port: u16,
    pub admin_token: String,
    pub partner_token: String,
    pub customer_token: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env_or("PORT", "3000").parse().unwrap_or(3000),
            admin_token: env_or("ADMIN_TOKEN", "admin-secret"),
            partner_token: env_or("PARTNER_TOKEN", "partner-secret"),
            customer_token: env_or("CUSTOMER_TOKEN", "customer-secret"),
        }
    }

    pub fn addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}
