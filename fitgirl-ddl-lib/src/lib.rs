use std::sync::OnceLock;

use nyquest::AsyncClient;

pub mod errors;
pub mod extract;
pub mod scrape;

pub static NYQUEST_CLIENT: OnceLock<AsyncClient> = OnceLock::new();
