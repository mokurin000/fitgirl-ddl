#![allow(unreachable_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs::OpenOptions};

use tracing_subscriber::EnvFilter;
use winio::prelude::App;

use crate::ui::main_model::MainModel;

mod utils;

mod ui {
    pub mod main_model;
    pub mod select_box;
}

pub mod model;

fn main() -> Result<(), Box<dyn Error>> {
    nyquest_preset::register();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .with_writer(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(concat!(env!("CARGO_PKG_NAME"), ".log"))?,
        )
        .init();

    App::new(env!("CARGO_PKG_NAME"))?.run::<MainModel>(())?;
    Ok(())
}
