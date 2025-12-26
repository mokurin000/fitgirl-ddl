use std::error::Error;

use chrono::{DateTime, Local};

use crate::search::SearchEntry;

pub fn process_time(rfc3339: &str) -> Result<DateTime<Local>, Box<dyn Error + Send + Sync>> {
    let dt = DateTime::parse_from_rfc3339(rfc3339).map_err(|_| "invalid time format")?;
    Ok(dt.to_utc().with_timezone(&Local))
}

pub fn display_table(
    i: impl IntoIterator<Item = SearchEntry>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use richrs::prelude::*;

    let mut c = Console::new();
    let mut table = Table::new().show_header(false);

    table.add_column(Column::empty());
    table.add_column(Column::empty());

    let column_width = 100;
    let max_attr_len = 5;
    let string_len_limit = column_width - 8 - max_attr_len;

    for SearchEntry { title, href, date } in i {
        let mut title = {
            let mut title = &*title;
            title = title.split_once(" – ").map(|p| p.0).unwrap_or(title);
            title = title.split_once(" - ").map(|p| p.0).unwrap_or(title);
            title.to_string()
        };
        if title.len() > string_len_limit {
            title = title.chars().take(string_len_limit - 3).collect::<String>() + "...";
        }

        table.add_row_cells(["date", &date]);
        table.add_row_cells(["game", &title]);
        table.add_row_cells(["link", &href]);
        table.add_section();
    }

    c.write_segments(&table.render(100))?;
    c.print("")?;

    Ok(())
}
