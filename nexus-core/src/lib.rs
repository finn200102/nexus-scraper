//! A novel scraper build in rust.
//!
pub mod models;
pub mod sites;
pub mod error;
pub mod parser;
pub mod network;

pub use network::{detect_site_from_url, parse_date};
