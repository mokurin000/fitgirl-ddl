use std::{collections::BTreeMap, fmt::Write};

use fitgirl_ddl_lib::set_fg_cookies;
use fitgirl_ddl_lib::{extract::DDL, init_nyquest};
use itertools::Itertools;
use spdlog::{debug, error, info, warn};

use compio::runtime::spawn;
use winio::{
    AsWindow, Button, Child, Component, ComponentSender, Enable, Layoutable, Margin,
    MaybeBorrowedWindow, MessageBox, MessageBoxButton, MessageBoxResponse, MessageBoxStyle,
    Progress, Size, StackPanel, TextBox, Visible, Window, WindowEvent,
};

use crate::model::Cookie;
use crate::ui::select_box::{SelectEvent, SelectWindow};
use crate::utils::{ExtractionInfo, export_ddl};
use crate::utils::{centralize_window, collect_groups};

#[allow(unused)]
pub(crate) struct MainModel {
    window: Child<Window>,
    selective_boxes: BTreeMap<usize, Child<SelectWindow>>,
    button: Child<Button>,
    url_edit: Child<TextBox>,
    progress: Child<Progress>,
    position: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum MainMessage {
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
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init(());
        window.set_text("fitgirl-ddl");
        window.set_size(Size::new(800.0, 130.0));

        centralize_window(&mut window);

        let url_edit = Child::<TextBox>::init(&window);
        let mut button = Child::<Button>::init(&window);
        button.set_text(" Scrape ");
        let mut progress = Child::<Progress>::init(&window);
        progress.set_range(0, 1);

        spawn(async {
            info!("init: nyquest");
            _ = init_nyquest().await;

            let cookies = match compio::fs::read("cookies.json").await {
                Err(e) => {
                    error!("failed to read cookies.json: {e}");
                    return;
                }
                Ok(bytes) => serde_json::from_slice::<Vec<Cookie>>(&bytes),
            };

            let cookies = match cookies {
                Err(e) => {
                    error!("failed to decode cookies.json: {e}");
                    return;
                }
                Ok(c) => c,
            };

            let _ = set_fg_cookies(
                cookies
                    .iter()
                    .map(|Cookie { name, value }| format!("{name}={value}"))
                    .join("; "),
            );
        })
        .detach();

        window.set_visible(true);

        Self {
            window,
            url_edit,
            button,
            progress,
            position: 0,
            selective_boxes: BTreeMap::default(),
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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
        let fut_swindows = self.selective_boxes.values_mut().map(|s| {
            s.start(
                sender,
                |e| match e {
                    SelectEvent::Update => Some(MainMessage::Redraw),
                    SelectEvent::Close(window_id) => Some(MainMessage::CloseSelective(window_id)),
                },
                || MainMessage::Redraw,
            )
        });
        let fut_tbox = self.url_edit.start(
            sender,
            |event| match event {
                winio::TextBoxEvent::Change => Some(MainMessage::Redraw),
                _ => None,
            },
            || MainMessage::Redraw,
        );

        futures_util::join!(
            fut_window,
            fut_button,
            fut_tbox,
            futures_util::future::join_all(fut_swindows)
        )
        .0
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
                self.button.enable();
                false
            }
            MainMessage::Download => {
                info!("start downloading!");

                let text = self.url_edit.text();
                if text.trim().is_empty() {
                    warn!("please enter URL first!");
                    return false;
                }

                let sender = sender.clone();

                self.button.disable();

                // reset range
                self.progress.set_range(0, 0);
                let selective = true;

                spawn(async move {
                    let urls = text.split([' ', '\n', '\t']).filter(|s| !s.is_empty());
                    let export = export_ddl(urls, 2, &sender, selective);

                    let (export_result, ..) = futures_util::join!(export);
                    match export_result {
                        Err(e) => {
                            popup_message(
                                (),
                                format!("failed to scrape: {e}"),
                                MessageBoxStyle::Error,
                            )
                            .await
                        }
                        Ok(ExtractionInfo {
                            missing_files,
                            scrape_errors,
                            ..
                        }) => {
                            let missing = missing_files.join("\n");
                            let errors = scrape_errors.join("\n");

                            let mut message = String::new();

                            if !missing.is_empty() {
                                _ = message.write_fmt(format_args!(
                                    "File Not Found Or Deleted:\n{missing}\n"
                                ));
                            }
                            if !errors.is_empty() {
                                _ = message.write_fmt(format_args!("Failed:\n{errors}"));
                            }

                            compio::runtime::spawn(async move {
                                let message = message.trim();
                                if !message.is_empty() {
                                    popup_message((), message.trim(), MessageBoxStyle::Warning)
                                        .await
                                }
                            })
                            .detach();
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
                let swindow = Child::<SelectWindow>::init((collect_groups(ddls), game_name));
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

        let mut layout_final = StackPanel::new(winio::Orient::Vertical);
        layout_final
            .push(&mut layout)
            .grow(true)
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
    parent: impl Into<MaybeBorrowedWindow<'_>>,
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
