#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fmt::Write};

use fitgirl_ddl_gui::{ExtractionInfo, export_ddl};
use fitgirl_ddl_lib::init_nyquest;
use spdlog::info;

use compio::runtime::spawn;
use winio::{
    App, AsWindow, Button, Child, Component, ComponentSender, Edit, Layoutable, MessageBox,
    MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Size, StackPanel, Window, WindowEvent,
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
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    Download,
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
        button.set_text("  提交  ");

        spawn(async {
            info!("init: nyquest");
            _ = init_nyquest().await;
        })
        .detach();

        Self {
            window,
            url_edit,
            button,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
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
        self.window.update().await;

        match message {
            MainMessage::Close => {
                match MessageBox::new()
                    .title(env!("CARGO_PKG_NAME"))
                    .message("Confirm Exit")
                    .instruction("Are you sure to exit fitgirl-ddl?")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .show(Some(&*self.window))
                    .await
                {
                    MessageBoxResponse::Yes => {
                        sender.output(());
                    }
                    _ => {}
                }
                false
            }
            MainMessage::Download => {
                let msgbox = popup_message(
                    Some(&*self.window),
                    "Started exporting direct links...",
                    MessageBoxStyle::Info,
                );

                let text = self.url_edit.text();
                let urls = text.split_whitespace().filter(|s| !s.is_empty());
                let export = export_ddl(urls, 2);

                let (export_result, ..) = futures_util::join!(export, msgbox);
                match export_result {
                    Err(e) => {
                        popup_message(
                            Some(&*self.window),
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
                            _ = message
                                .write_fmt(format_args!("File Not Found Or Deleted:\n{missing}\n"));
                        }
                        if !errors.is_empty() {
                            _ = message.write_fmt(format_args!("Failed:\n{errors}\n"));
                        }

                        popup_message(Some(&*self.window), message, MessageBoxStyle::Info).await
                    }
                }
                false
            }
            MainMessage::Redraw => true,
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let mut layout = StackPanel::new(winio::Orient::Horizontal);
        layout.push(&mut self.url_edit).grow(true).finish();
        layout.push(&mut self.button).finish();
        layout.set_size(self.window.client_size());
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
