use std::error::Error;

use fitgirl_ddl_lib::init_nyquest;
use spdlog::info;

use std::time::Duration;

use compio::{runtime::spawn, time::interval};
use winio::{
    App, BrushPen, Canvas, CanvasEvent, Child, Color, ColorTheme, Component, ComponentSender,
    DrawingFontBuilder, Grid, HAlign, Layoutable, MessageBox, MessageBoxButton, MessageBoxResponse,
    MessageBoxStyle, MouseButton, Point, Rect, Size, SolidColorBrush, VAlign, Window, WindowEvent,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    nyquest_preset::register();

    let mut runtime = App::new();
    runtime.block_on(async {
        info!("init: nyquest");
        _ = init_nyquest().await;
    });
    runtime.run::<MainModel>(0, &());
    Ok(())
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    counter: usize,
}

#[derive(Debug)]
enum MainMessage {
    Tick,
    Close,
    Redraw,
    Mouse(MouseButton),
    MouseMove(Point),
}

impl Component for MainModel {
    type Event = ();
    type Init = usize;
    type Message = MainMessage;
    type Root = ();

    fn init(counter: Self::Init, _root: &Self::Root, sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());
        let canvas = Child::<Canvas>::init((), &window);

        window.set_text("Basic example");
        window.set_size(Size::new(800.0, 600.0));

        let sender = sender.clone();
        spawn(async move {
            info!("setup: ticker");
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                info!("ticking");
                if !sender.post(MainMessage::Tick) {
                    info!("ticking break");
                    break;
                }
            }
        })
        .detach();
        Self {
            window,
            canvas,
            counter,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Resize => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_canvas = self.canvas.start(sender, |e| match e {
            CanvasEvent::Redraw => Some(MainMessage::Redraw),
            CanvasEvent::MouseDown(b) | CanvasEvent::MouseUp(b) => Some(MainMessage::Mouse(b)),
            CanvasEvent::MouseMove(p) => Some(MainMessage::MouseMove(p)),
            _ => None,
        });
        futures_util::future::join(fut_window, fut_canvas).await;
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.canvas.update()).await;
        match message {
            MainMessage::Tick => {
                self.counter += 1;
                true
            }
            MainMessage::Close => {
                match MessageBox::new()
                    .title("fitgirl-ddl")
                    .message("确认退出")
                    .instruction("即将退出程序")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .show(Some(&*self.window))
                    .await
                {
                    MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                        sender.output(());
                    }
                    _ => {}
                }
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Mouse(_b) => {
                info!("{:?}", _b);
                false
            }
            MainMessage::MouseMove(_p) => {
                info!("{:?}", _p);
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        self.canvas.render();

        let csize = self.window.client_size();
        {
            let mut grid = Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap();
            grid.push(&mut self.canvas).column(1).row(1).finish();
            grid.set_size(csize);
        }

        let size = self.canvas.size();
        let is_dark = ColorTheme::current() == ColorTheme::Dark;
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        let mut ctx = self.canvas.context();
        ctx.draw_ellipse(
            BrushPen::new(brush.clone(), 1.0),
            Rect::new((size.to_vector() / 4.0).to_point(), size / 2.0),
        );
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .family("Arial")
                .size(12.0)
                .build(),
            (size.to_vector() / 2.0).to_point(),
            format!("Hello world!\nRunning: {}s", self.counter),
        );
    }
}
