#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::BTreeMap, error::Error, fmt::Write, sync::Arc};

use fitgirl_ddl_lib::{extract::DDL, init_nyquest};
use spdlog::{Level, debug, info, sink::FileSink};

mod utils;
use utils::{ExtractionInfo, export_ddl};
mod select_box;

use compio::runtime::spawn;
use winio::{
    App, AsWindow, Button, CheckBox, Child, Component, ComponentSender, Layoutable, Margin,
    MessageBox, MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Progress, Size, StackPanel,
    TextBox, Visible, Window, WindowEvent,
};

use crate::select_box::{SelectWindow, collect_groups};

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

    App::new().run::<MainModel>((), &());
    Ok(())
}

#[allow(unused)]
struct MainModel {
    window: Child<Window>,
    selective_boxes: BTreeMap<usize, Child<SelectWindow>>,
    button: Child<Button>,
    url_edit: Child<TextBox>,
    progress: Child<Progress>,
    selective_download: Child<CheckBox>,
    downloading: bool,
    position: usize,
}

#[derive(Debug, Clone)]
enum MainMessage {
    Close,
    Redraw,
    Download,
    DownloadDone,
    IncreaseCount,
    SetMaxCap(usize),
    CreateSelection(Vec<DDL>, String),
    CloseSelective(usize),
}

impl Component for MainModel {
    type Event = ();
    type Init = ();
    type Message = MainMessage;
    type Root = ();

    fn init(_: Self::Init, _root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());
        window.set_text("fitgirl-ddl");
        window.set_size(Size::new(800.0, 130.0));

        let url_edit = Child::<TextBox>::init((), &window);
        let mut button = Child::<Button>::init((), &window);
        button.set_text(" Submit ");
        let mut progress = Child::<Progress>::init((), &window);
        progress.set_range(0, 1);
        let mut selective_download = Child::<CheckBox>::init((), &window);
        selective_download.set_text("Selective");

        spawn(async {
            info!("init: nyquest");
            _ = init_nyquest().await;
        })
        .detach();

        window.set_visible(true);

        Self {
            window,
            url_edit,
            button,
            progress,
            selective_download,
            downloading: false,
            position: 0,
            selective_boxes: BTreeMap::default(),
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let window = &mut self.window;
        let fut_window = window.start(
            sender,
            |e| match e {
                WindowEvent::Close => Some(MainMessage::Close),
                WindowEvent::Resize => Some(MainMessage::Redraw),
                _ => None,
            },
            || MainMessage::Redraw,
        );
        let fut_button = self.button.start(
            sender,
            |e| match e {
                winio::ButtonEvent::Click => Some(MainMessage::Download),
                _ => None,
            },
            || MainMessage::Redraw,
        );
        let fut_cbox = self
            .selective_download
            .start(sender, |_| None, || MainMessage::Redraw);
        let fut_swindows = self.selective_boxes.values_mut().map(|s| {
            s.start(
                sender,
                |e| match e {
                    select_box::SelectEvent::Update => Some(MainMessage::Redraw),
                    select_box::SelectEvent::Close(window_id) => {
                        Some(MainMessage::CloseSelective(window_id))
                    }
                },
                || MainMessage::Redraw,
            )
        });

        futures_util::join!(
            fut_window,
            fut_button,
            fut_cbox,
            futures_util::future::join_all(fut_swindows)
        );
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        self.window.update().await;
        let sub_update = futures_util::future::join_all(
            self.selective_boxes
                .values_mut()
                .map(async |sbox| sbox.update().await),
        )
        .await
        .into_iter()
        .any(|b| b);

        (match message {
            MainMessage::Close => {
                if MessageBox::new()
                    .title(env!("CARGO_PKG_NAME"))
                    .message("Confirm Exit")
                    .instruction("Are you sure to exit fitgirl-ddl?")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .show(Some(self.window.as_window()))
                    .await
                    == MessageBoxResponse::Yes
                {
                    sender.output(());
                }
                false
            }
            MainMessage::DownloadDone => {
                self.downloading = false;
                false
            }
            MainMessage::Download => {
                if self.downloading {
                    return false;
                }

                let text = self.url_edit.text();
                if text.trim().is_empty() {
                    return false;
                }

                let sender = sender.clone();

                self.downloading = true;

                // reset range
                self.progress.set_range(0, 0);
                let selective = self.selective_download.is_checked();

                spawn(async move {
                    let urls = text.split([' ', '\n', '\t']).filter(|s| !s.is_empty());
                    let export = export_ddl(urls, 2, &sender, selective);

                    let (export_result, ..) = futures_util::join!(export);
                    match export_result {
                        Err(e) => {
                            popup_message(
                                Option::<Window>::None,
                                format!("failed to scrape: {e}"),
                                MessageBoxStyle::Error,
                            )
                            .await
                        }
                        Ok(ExtractionInfo {
                            saved_files,
                            missing_files,
                            scrape_errors,
                        }) => {
                            let exported = saved_files.join("\n");
                            let missing = missing_files.join("\n");
                            let errors = scrape_errors.join("\n");

                            let mut message = String::new();

                            if !exported.is_empty() {
                                _ = message.write_fmt(format_args!("Exported:\n{exported}\n"));
                            }
                            if !missing.is_empty() {
                                _ = message.write_fmt(format_args!(
                                    "File Not Found Or Deleted:\n{missing}\n"
                                ));
                            }
                            if !errors.is_empty() {
                                _ = message.write_fmt(format_args!("Failed:\n{errors}\n"));
                            }

                            let message = message.trim();
                            info!("{message}");
                        }
                    }

                    sender.post(MainMessage::DownloadDone);
                })
                .detach();

                false
            }
            MainMessage::Redraw => true,
            MainMessage::IncreaseCount => {
                self.position += 1;

                debug!("received increasement! new pos: {}", self.position);
                self.progress.set_pos(self.position);

                false
            }
            MainMessage::SetMaxCap(new) => {
                debug!("received max capacity! new cap: {new}");

                self.progress.set_range(0, new);
                self.progress.set_pos(0);
                self.position = 0;

                false
            }
            MainMessage::CreateSelection(ddls, game_name) => {
                let swindow = Child::<SelectWindow>::init((collect_groups(ddls), game_name), &());
                let window_id = swindow.window_id;

                self.selective_boxes.insert(window_id, swindow);
                false
            }
            MainMessage::CloseSelective(id) => {
                self.selective_boxes.remove_entry(&id);
                false
            }
        } || sub_update)
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        for sbox in self.selective_boxes.values_mut() {
            sbox.render();
        }

        let mut layout = StackPanel::new(winio::Orient::Horizontal);
        layout.push(&mut self.url_edit).grow(true).finish();
        layout.push(&mut self.button).finish();

        let mut layout2 = StackPanel::new(winio::Orient::Horizontal);
        layout2
            .push(&mut self.selective_download)
            .grow(true)
            .finish();

        let mut layout_final = StackPanel::new(winio::Orient::Vertical);
        layout_final
            .push(&mut layout)
            .grow(true)
            .margin(Margin::new(5., 5., 5., 5.))
            .finish();
        layout_final
            .push(&mut layout2)
            .margin(Margin::new(5., 5., 5., 5.))
            .finish();
        layout_final
            .push(&mut self.progress)
            .margin(Margin::new(5., 5., 5., 5.))
            .finish();
        layout_final.set_size(self.window.client_size());
    }
}

async fn popup_message(
    parent: Option<impl AsWindow>,
    message: impl AsRef<str>,
    level: MessageBoxStyle,
) {
    MessageBox::new()
        .title(env!("CARGO_PKG_NAME"))
        .message(message)
        .style(level)
        .buttons(MessageBoxButton::Ok)
        .show(parent)
        .await;
}
