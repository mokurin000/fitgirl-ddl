#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::Arc};

use spdlog::{Level, sink::FileSink};
use winio::App;

use crate::ui::main_model::MainModel;

mod utils;

mod ui {
    pub mod main_model;
    pub mod select_box;
}

pub mod model;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    nyquest_preset::register();

    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::MoreSevereEqual(
        if cfg!(debug_assertions) {
            Level::Debug
        } else {
            Level::Info
        },
    ));
    let new_logger = spdlog::default_logger().fork_with(|log| {
        let file_sink = Arc::new(
            FileSink::builder()
                .path(concat!(env!("CARGO_PKG_NAME"), ".log"))
                .build()?,
        );
        log.sinks_mut().push(file_sink);
        Ok(())
    })?;
    spdlog::set_default_logger(new_logger);

    App::new().run::<MainModel>(());
    Ok(())
}
