pub mod ui {
    pub mod main_model;
    pub mod select_box;
}

pub type Result<T> = std::result::Result<T, color_eyre::Report>;

pub mod model;

pub mod utils;
