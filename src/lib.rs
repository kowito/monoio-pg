pub mod auth;
pub mod client;
pub mod codec;
pub mod connection;
pub mod error;
pub mod pool;

pub use client::Client;
pub use error::{Error, Result};
pub use pool::Pool;
