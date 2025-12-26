use std::error::Error;

use chrono::{DateTime, Local};

pub fn process_time(rfc3339: &str) -> Result<DateTime<Local>, Box<dyn Error + Send + Sync>> {
    let dt = DateTime::parse_from_rfc3339(rfc3339).map_err(|_| "invalid time format")?;
    Ok(dt.to_utc().with_timezone(&Local))
}
