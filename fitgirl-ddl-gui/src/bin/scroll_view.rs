use ahash::AHashMap;
use winio::prelude::*;

use fitgirl_ddl_gui::Result;
use fitgirl_ddl_gui::ui::select_box::SelectWindow;

fn main() -> Result<()> {
    App::builder()
        .name("scrollview-test")
        .build()?
        .block_on(SelectWindow::run_until_event((
            AHashMap::from_iter((0..100).map(|id| (format!("group {id:03}"), vec![]))),
            "scrollview-test".into(),
        )))?;

    Ok(())
}
