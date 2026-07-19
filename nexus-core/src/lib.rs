//! A novel scraper build in rust.
//!
pub mod error;
pub mod models;
pub mod network;
pub mod parser;
pub mod sites;

pub use network::{create_http_client, detect_site_from_url, parse_date};
