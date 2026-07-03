#![allow(dead_code)] // mute rust-analyzer false-positive
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;

use tracing_subscriber::EnvFilter;
use winio::prelude::App;
use winio::ui::ComponentExt;

use fitgirl_ddl_gui::Result;
use fitgirl_ddl_gui::ui::main_model::MainModel;

fn main() -> Result<()> {
    color_eyre::install()?;

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

    App::builder()
        .name(env!("CARGO_PKG_NAME"))
        .build()?
        .block_on(MainModel::run_until_event(()))?;
    Ok(())
}
