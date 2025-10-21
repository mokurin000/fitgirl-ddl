use std::sync::atomic::AtomicUsize;

use ahash::AHashMap;
use fitgirl_ddl_lib::extract::DDL;
use itertools::Itertools;
use winio::prelude::*;
use tracing::debug;

use crate::utils::{centralize_window, write_aria2_input};

#[derive(Debug)]
pub struct SelectWindow {
    pub window_id: usize,

    pub window: Child<Window>,
    pub scroll: Child<ScrollView>,
    pub checkbox: Vec<Child<CheckBox>>,
    pub submit: Child<Button>,

    pub game_name: String,
    pub groups: AHashMap<String, Vec<DDL>>,
}

#[derive(Debug, Clone)]
pub enum SelectMessage {
    Noop,
    CloseWindow,
    Refresh,
    SaveFile,
}

#[derive(Debug, Clone)]
pub enum SelectEvent {
    Close(usize),
}
static SWINDOW_ID: AtomicUsize = AtomicUsize::new(0);

impl Component for SelectWindow {
    type Init<'a> = (AHashMap<String, Vec<DDL>>, String);
    type Message = SelectMessage;
    type Event = SelectEvent;

    fn init((groups, game_name): Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: &game_name,
                size: Size::new(500., 500.),
            },
            scroll: ScrollView = (&window) => {
                vscroll: true,
                hscroll: false,
            },
            submit: Button = (&window) => {
                text: "Confirm",
            },
        }

        centralize_window(&mut window);

        let mut checkbox = Vec::with_capacity(groups.len());
        for group_name in groups.keys().sorted() {
            init! {
                cbox: CheckBox = (&scroll) => {
                    text: group_name,
                    checked: ["fitgirl-repacks.site", "FIXED"]
                                .iter()
                                .any(|keyword| group_name.contains(keyword))
                },
            }

            checkbox.push(cbox);
        }

        window.show();

        sender.post(SelectMessage::Refresh);

        Self {
            window_id: SWINDOW_ID.fetch_add(1, std::sync::atomic::Ordering::AcqRel),
            window,
            scroll,
            checkbox,
            submit,
            groups,
            game_name,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_widgets = async {
            start! {
                sender, default: SelectMessage::Noop,
                self.window => {
                    WindowEvent::Close => SelectMessage::CloseWindow,
                    WindowEvent::Resize => SelectMessage::Refresh,
                },
                self.submit => {
                    ButtonEvent::Click => {
                        SelectMessage::SaveFile
                    },
                },
                self.scroll => {},
            }
        };
        let fut_cboxes = self
            .checkbox
            .iter_mut()
            .map(async |c| {
                c.start(sender, |_| None, || SelectMessage::Noop).await;
            })
            .collect::<Vec<_>>();

        futures_util::join!(fut_widgets, futures_util::future::join_all(fut_cboxes)).0
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        debug!("SelectWindow [update]: {message:?}");

        let mut needs_render = self.scroll.update().await;
        needs_render |= match message {
            SelectMessage::Noop => false,
            SelectMessage::CloseWindow => {
                sender.output(SelectEvent::Close(self.window_id));
                false
            }
            SelectMessage::Refresh => {
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
        };
        needs_render
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let mut layout_out = layout! {
            StackPanel::new(Orient::Vertical),
            self.scroll => { grow: true, margin: Margin::new_all_same(5.) },
            self.submit => { margin: Margin::new_all_same(5.) },
        };

        layout_out.set_size(self.window.client_size());

        let mut cboxes = StackPanel::new(Orient::Vertical);
        for cbox in &mut self.checkbox {
            cboxes.push(cbox).margin(Margin::new_all_same(5.)).finish();
        }
        cboxes.set_size(self.scroll.size());
    }
}
