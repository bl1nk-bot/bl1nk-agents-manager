// Config module - aggregates all configuration-related modules

pub mod config;
pub mod extra_types;
pub mod handler;
pub mod loader;
pub mod schema;

// Re-export commonly used items from config.rs for convenience
pub use config::Config;
pub use config::RoutingConfig;
pub use config::RoutingRule;
pub use config::RoutingTier;
pub use config::RateLimitingConfig;

// Re-export from other modules
pub use handler::{ConfigContext, ConfigHandler, ConfigHandlerDeps};
pub use loader::*;
pub use schema::*;