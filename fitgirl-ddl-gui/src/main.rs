#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fmt::Write};

use fitgirl_ddl_lib::init_nyquest;
use spdlog::info;

mod utils;
use utils::{ExtractionInfo, export_ddl};

use compio::runtime::spawn;
use winio::{
    App, AsWindow, Button, Child, Component, ComponentSender, Edit, Layoutable, MessageBox,
    MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Progress, Size, StackPanel, Window,
    WindowEvent,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    nyquest_preset::register();

    App::new().run::<MainModel>((), &());
    Ok(())
}

#[allow(unused)]
struct MainModel {
    window: Child<Window>,
    button: Child<Button>,
    url_edit: Child<Edit>,
    progress: Child<Progress>,
    downloading: bool,
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    Download,
    DownloadDone,
    IncreaseCount,
    SetMaxCap(usize),
}

impl Component for MainModel {
    type Event = ();
    type Init = ();
    type Message = MainMessage;
    type Root = ();

    fn init(_: Self::Init, _root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());

        window.set_text("fitgirl-ddl");
        window.set_size(Size::new(800.0, 80.0));

        let url_edit = Child::<Edit>::init((), &window);
        let mut button = Child::<Button>::init((), &window);
        button.set_text("Submit");
        let mut progress = Child::<Progress>::init((), &window);
        progress.set_range(0, 0);

        spawn(async {
            info!("init: nyquest");
            _ = init_nyquest().await;
        })
        .detach();

        Self {
            window,
            url_edit,
            button,
            progress,
            downloading: false,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let window = &mut self.window;
        let fut_window = window.start(sender, |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Resize => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_button = self.button.start(sender, |e| match e {
            winio::ButtonEvent::Click => Some(MainMessage::Download),
            _ => unimplemented!(),
        });

        futures_util::join!(fut_window, fut_button,);
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        {
            self.window.update().await;
        }

        match message {
            MainMessage::Close => {
                match MessageBox::new()
                    .title(env!("CARGO_PKG_NAME"))
                    .message("Confirm Exit")
                    .instruction("Are you sure to exit fitgirl-ddl?")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .show(Some(self.window.as_window()))
                    .await
                {
                    MessageBoxResponse::Yes => {
                        sender.output(());
                    }
                    _ => {}
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
                let sender = sender.clone();

                self.downloading = true;

                // reset range
                self.progress.set_range(0, 0);

                _ = spawn(async move {
                    let urls = text.split([' ', '\n', '\t']).filter(|s| !s.is_empty());
                    let export = export_ddl(urls, 2, &sender);

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

                            popup_message(Option::<Window>::None, message.trim(), MessageBoxStyle::Info)
                                .await
                        }
                    }

                    sender.post(MainMessage::DownloadDone);
                })
                .detach();

                false
            }
            MainMessage::Redraw => true,
            MainMessage::IncreaseCount => {
                let pos = self.progress.pos() + 1;
                self.progress.set_pos(pos);
                false
            }
            MainMessage::SetMaxCap(new) => {
                self.progress.set_range(0, new);
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let mut layout = StackPanel::new(winio::Orient::Horizontal);
        layout.push(&mut self.url_edit).grow(true).finish();
        layout.push(&mut self.button).finish();

        let mut layout_final = StackPanel::new(winio::Orient::Vertical);
        layout_final.push(&mut layout).grow(true).finish();
        layout_final.push(&mut self.progress).finish();
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
