use std::{collections::BTreeMap, fmt::Write};

use fitgirl_ddl_lib::set_fg_cookies;
use fitgirl_ddl_lib::{extract::DDL, init_nyquest};
use itertools::Itertools;
use tracing::{debug, error, info, warn};

use compio::runtime::spawn;
use winio::prelude::*;

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
    Noop,
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
        init! {
            window: Window = (()) => {
                text: "fitgirl-ddl",
                size: Size::new(800., 130.),
            },
            url_edit: TextBox = (&window),
            button: Button = (&window) => {
                text: " Scrape ",
            },
            progress: Progress = (&window) => {
                minimum: 0,
                maximum: 1,
            },
        }

        centralize_window(&mut window);

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

        window.show();

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
        let fut_widgets = async {
            start! {
                sender, default: MainMessage::Noop,
                self.window => {
                    WindowEvent::Close => MainMessage::Close,
                    WindowEvent::Resize => MainMessage::Redraw,
                },
                self.button => {
                    ButtonEvent::Click => MainMessage::Download,
                },
                self.url_edit => {
                    TextBoxEvent::Change => MainMessage::Redraw,
                },
            }
        };
        let fut_swindows = self.selective_boxes.values_mut().map(|s| async {
            start! {
                sender, default: MainMessage::Noop,
                s => {
                    SelectEvent::Update => MainMessage::Redraw,
                    SelectEvent::Close(window_id) => MainMessage::CloseSelective(window_id),
                },
            }
        });

        futures_util::join!(fut_widgets, futures_util::future::join_all(fut_swindows)).0
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
            MainMessage::Noop => false,
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
                self.progress.set_pos(0);
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

                self.progress.set_maximum(new);
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

        let mut layout = layout! {
            StackPanel::new(Orient::Horizontal),
            self.url_edit => { grow: true },
            self.button,
        };
        let mut layout_final = layout! {
            StackPanel::new(Orient::Vertical),
            layout => { grow: true, margin: Margin::new_all_same(5.) },
            self.progress => { margin: Margin::new_all_same(5.) },
        };

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
