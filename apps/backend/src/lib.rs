pub mod adapters;
pub mod app_state;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use adapters::inbound::http::router;
