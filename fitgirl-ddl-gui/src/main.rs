use std::error::Error;

use fitgirl_ddl_gui::export_ddl;
use fitgirl_ddl_lib::init_nyquest;
use spdlog::info;

use compio::runtime::spawn;
use winio::{
    App, Button, Child, Component, ComponentSender, Edit, Layoutable, MessageBox, MessageBoxButton,
    MessageBoxResponse, MessageBoxStyle, Size, StackPanel, Window, WindowEvent,
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
                    .title("fitgirl-ddl")
                    .message("Confirm Exit")
                    .instruction("This will terminate existing export task!")
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
                let msgbox = MessageBox::new()
                    .title("fitgirl-ddl")
                    .message("Started exporting direct links...")
                    .style(MessageBoxStyle::Info)
                    .show(Some(&*self.window));

                let text = self.url_edit.text();
                let urls = text.split_whitespace().filter(|s| !s.is_empty());
                let export = export_ddl(urls, 2);

                futures_util::join!(export, msgbox);
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
