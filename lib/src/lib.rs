pub mod error;
pub mod models;
pub mod output;
pub mod colorize;
pub mod visitors;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;