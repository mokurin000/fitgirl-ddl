use std::sync::atomic::AtomicUsize;

use ahash::AHashMap;
use fitgirl_ddl_lib::extract::DDL;
use itertools::Itertools;
use winio::{
    Button, CheckBox, Child, Component, Enable, Layoutable, Margin, Size, StackPanel, Visible,
    Window,
};

use crate::utils::{centralize_window, write_aria2_input};

#[derive(Debug)]
pub struct SelectWindow {
    pub window_id: usize,

    pub window: Child<Window>,
    pub checkbox: Vec<Child<CheckBox>>,
    pub submit: Child<Button>,

    pub game_name: String,
    pub groups: AHashMap<String, Vec<DDL>>,
}

#[derive(Debug, Clone)]
pub enum SelectMessage {
    CloseWindow,
    Refresh,
    SaveFile,
}

#[derive(Debug, Clone)]
pub enum SelectEvent {
    Update,
    Close(usize),
}
static SWINDOW_ID: AtomicUsize = AtomicUsize::new(0);

impl Component for SelectWindow {
    type Init<'a> = (AHashMap<String, Vec<DDL>>, String);
    type Message = SelectMessage;
    type Event = SelectEvent;

    fn init((groups, game_name): Self::Init<'_>, sender: &winio::ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init(());
        window.set_text(&game_name);
        window.set_size(Size::new(500., 500.));

        centralize_window(&mut window);

        let mut checkbox = Vec::with_capacity(groups.len());
        for group_name in groups.keys().sorted() {
            let mut cbox = Child::<CheckBox>::init(&window);
            cbox.set_text(group_name);

            if ["optional", "selective"]
                .iter()
                .all(|keyword| !group_name.contains(keyword))
            {
                cbox.set_checked(true);
                cbox.disable();
            }

            checkbox.push(cbox);
        }

        let mut submit = Child::<Button>::init(&window);
        submit.set_text("Confirm");

        window.set_visible(true);

        sender.post(SelectMessage::Refresh);

        Self {
            window_id: SWINDOW_ID.fetch_add(1, std::sync::atomic::Ordering::AcqRel),
            window,
            checkbox,
            submit,
            groups,
            game_name,
        }
    }

    async fn start(&mut self, sender: &winio::ComponentSender<Self>) {
        let fut_window = self.window.start(
            sender,
            |e| match e {
                winio::WindowEvent::Close => Some(SelectMessage::CloseWindow),
                winio::WindowEvent::Resize => Some(SelectMessage::Refresh),
                winio::WindowEvent::Move => Some(SelectMessage::Refresh),
                _ => None,
            },
            || SelectMessage::Refresh,
        );
        let fut_submit = self.submit.start(
            sender,
            |e| match e {
                winio::ButtonEvent::Click => {
                    sender.output(SelectEvent::Update);
                    Some(SelectMessage::SaveFile)
                }
                _ => None,
            },
            || SelectMessage::Refresh,
        );
        let fut_cboxes = self
            .checkbox
            .iter_mut()
            .map(async |c| {
                c.start(sender, |_| None, || SelectMessage::Refresh).await;
            })
            .collect::<Vec<_>>();

        futures_util::join!(
            fut_window,
            fut_submit,
            futures_util::future::join_all(fut_cboxes)
        );
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &winio::ComponentSender<Self>,
    ) -> bool {
        match message {
            SelectMessage::CloseWindow => {
                sender.output(SelectEvent::Close(self.window_id));
                false
            }
            SelectMessage::Refresh => {
                sender.output(SelectEvent::Update);
                true
            }
            SelectMessage::SaveFile => {
                let ddls: Vec<_> = self
                    .checkbox
                    .iter()
                    .filter(|c| c.is_checked())
                    .map(|c| c.text())
                    .filter_map(|t| self.groups.get(&t))
                    .flatten()
                    .collect();

                write_aria2_input(ddls, format!("{}.txt", self.game_name)).await;
                false
            }
        }
    }

    fn render(&mut self, _sender: &winio::ComponentSender<Self>) {
        self.window.render();

        let mut layout_out = StackPanel::new(winio::Orient::Vertical);
        let mut layout = StackPanel::new(winio::Orient::Vertical);
        for cbox in &mut self.checkbox {
            layout
                .push(cbox)
                .margin(Margin::new(5., 5., 5., 5.))
                .finish();
        }

        layout_out
            .push(&mut layout)
            .grow(true)
            .margin(Margin::new(5., 5., 5., 5.))
            .finish();
        layout_out
            .push(&mut self.submit)
            .grow(false)
            .margin(Margin::new(5., 5., 5., 5.))
            .finish();

        layout_out.set_size(self.window.client_size());
    }
}
