pub mod error;
pub mod models;
pub mod output;
pub mod colorize;
pub mod visitors;
pub mod analyzer;
pub mod type_errors;
pub mod symbols;
pub mod dependencies;
pub mod tests;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
