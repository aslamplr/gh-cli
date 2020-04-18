pub mod core;
mod utils;

pub type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error + Send + Sync>>;
