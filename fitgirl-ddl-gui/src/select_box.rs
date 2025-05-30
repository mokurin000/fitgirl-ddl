use ahash::AHashMap;
use fitgirl_ddl_lib::extract::DDL;
use winio::{Button, CheckBox, Child, Component, Layoutable, Size, StackPanel, Window};

use crate::utils::write_aria2_input;

#[derive(Debug)]
pub struct SelectWindow {
    pub window: Child<Window>,
    pub checkbox: Vec<Child<CheckBox>>,
    pub submit: Child<Button>,

    pub game_name: String,
    pub groups: AHashMap<String, Vec<DDL>>,
}

#[derive(Debug)]
pub enum SelectMessage {
    CloseWindow,
    Refresh,
    SaveFile,
}

pub fn collect_groups(ddls: impl IntoIterator<Item = DDL>) -> AHashMap<String, Vec<DDL>> {
    let mut groups: AHashMap<String, Vec<DDL>> = AHashMap::new();

    for DDL {
        filename,
        direct_link,
    } in ddls
    {
        let group_name = filename
            .split_once(".part")
            .map(|(pre, _)| pre.to_string())
            .unwrap_or(filename.clone());
        groups.entry(group_name).or_default().push(DDL {
            filename,
            direct_link,
        });
    }

    groups
}

impl Component for SelectWindow {
    type Init = (AHashMap<String, Vec<DDL>>, String);
    type Root = ();
    type Message = SelectMessage;
    type Event = ();

    fn init(
        (groups, game_name): Self::Init,
        root: &Self::Root,
        _sender: &winio::ComponentSender<Self>,
    ) -> Self {
        let mut window = Child::<Window>::init((), &root);
        window.set_text(&game_name);
        window.set_size(Size::new(500., 500.));

        let mut checkbox = Vec::with_capacity(groups.len());
        for group_name in groups.keys() {
            let mut cbox = Child::<CheckBox>::init((), &window);
            cbox.set_text(group_name);

            if ["optional", "selective"]
                .iter()
                .all(|keyword| !group_name.contains(keyword))
            {
                cbox.set_checked(true);
            }

            checkbox.push(cbox);
        }

        let mut submit = Child::<Button>::init((), &window);
        submit.set_text("Confirm");

        Self {
            window,
            checkbox,
            submit,
            groups,
            game_name,
        }
    }

    async fn start(&mut self, sender: &winio::ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
            winio::WindowEvent::Close => Some(SelectMessage::CloseWindow),
            winio::WindowEvent::Resize => Some(SelectMessage::Refresh),
            _ => None,
        });
        let fut_submit = self.submit.start(sender, |e| match e {
            winio::ButtonEvent::Click => {
                sender.output(());
                Some(SelectMessage::SaveFile)
            }
            _ => None,
        });
        futures_util::join!(fut_window, fut_submit);
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &winio::ComponentSender<Self>,
    ) -> bool {
        match message {
            SelectMessage::CloseWindow => {
                sender.output(());
                false
            }
            SelectMessage::Refresh => true,
            SelectMessage::SaveFile => {
                let ddls: Vec<_> = self
                    .checkbox
                    .iter()
                    .filter(|c| c.is_checked())
                    .map(|c| c.text())
                    .filter_map(|t| self.groups.get(&t))
                    .flatten()
                    .collect();

                write_aria2_input(ddls, format!("{}_selected.txt", self.game_name)).await;
                false
            }
        }
    }

    fn render(&mut self, _sender: &winio::ComponentSender<Self>) {
        self.window.render();

        let mut layout_out = StackPanel::new(winio::Orient::Vertical);
        let mut layout = StackPanel::new(winio::Orient::Vertical);
        for cbox in &mut self.checkbox {
            layout.push(cbox).finish();
        }

        layout_out.push(&mut layout).grow(true).finish();
        layout_out.push(&mut self.submit).grow(false).finish();

        self.window.set_size(layout_out.preferred_size());
        layout_out.set_size(self.window.client_size());
    }
}
