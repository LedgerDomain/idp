#![allow(unused)]

use crate::{Message, PlumTableView, PlumView};
use iced::alignment;
use iced::theme;
use iced::widget::{
    checkbox, column, container, horizontal_space, image, radio, row, scrollable, slider, text,
    text_input, toggler, vertical_space,
};
use iced::widget::{Button, Column, Container, Slider};
use iced::Theme;
use iced::{Color, Element, Font, Length, Renderer, Sandbox};
use idp_core::BranchNode;
use idp_proto::{ContentEncoding, ContentFormat, Nonce, PlumBuilder, UnixNanoseconds};

pub struct App {
    plum_table_view: PlumTableView,
    // NOTE: Later this will be a generic View object
    view_stack_v: Vec<PlumView>,
    debug: bool,
    datahost: idp_core::Datahost,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        use pollster::FutureExt;
        // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
        let datahost_storage =
            idp_datahost_storage_sqlite::DatahostStorageSQLite::connect_and_run_migrations(
                "sqlite:idp-gui.db?mode=rwc",
            )
            .block_on()
            .expect("handle error");
        let datahost = idp_core::Datahost::open(datahost_storage);

        // // Generate a random Plum
        // let plum_head_seal = datahost
        //     .store_plum(
        //         &idp_proto::PlumBuilder::new()
        //             .with_plum_metadata_nonce(Nonce::generate())
        //             .with_plum_created_at(UnixNanoseconds::now())
        //             .with_plum_relations_and_plum_body_content_from(
        //                 &"Hippos are super cool and rad".to_string(),
        //                 None,
        //                 idp_proto::ContentEncoding::none(),
        //             )
        //             .unwrap()
        //             .build()
        //             .unwrap(),
        //         None,
        //     )
        //     .block_on()
        //     .unwrap();

        // Make some BranchNode content
        {
            let content_1 = "splunges are cool".to_string();
            let content_2 = "HIPPOS are cool".to_string();

            let content_1_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &content_1,
                    Some(&ContentFormat::charset_us_ascii()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let content_2_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &content_2,
                    Some(&ContentFormat::charset_us_ascii()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");

            let metadata_0_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &"Branch root".to_string(),
                    Some(&ContentFormat::charset_us_ascii()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let metadata_1_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &"Initial statement".to_string(),
                    Some(&ContentFormat::charset_us_ascii()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let metadata_2_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &"Revised statement authored by the HIPPO lobby".to_string(),
                    Some(&ContentFormat::charset_us_ascii()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");

            let content_1_plum_head_seal = datahost
                .store_plum(&content_1_plum, None)
                .block_on()
                .expect("pass");
            let content_2_plum_head_seal = datahost
                .store_plum(&content_2_plum, None)
                .block_on()
                .expect("pass");

            let metadata_0_plum_head_seal = datahost
                .store_plum(&metadata_0_plum, None)
                .block_on()
                .expect("pass");
            let metadata_1_plum_head_seal = datahost
                .store_plum(&metadata_1_plum, None)
                .block_on()
                .expect("pass");
            let metadata_2_plum_head_seal = datahost
                .store_plum(&metadata_2_plum, None)
                .block_on()
                .expect("pass");

            let branch_node_0 = BranchNode {
                ancestor_o: None,
                height: 0,
                metadata: metadata_0_plum_head_seal.clone(),
                content_o: None,
                posi_diff_o: None,
                nega_diff_o: None,
            };
            let branch_node_0_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &branch_node_0,
                    Some(&ContentFormat::json()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let branch_node_0_plum_head_seal = datahost
                .store_plum(&branch_node_0_plum, None)
                .block_on()
                .expect("pass");

            let branch_node_1 = BranchNode {
                ancestor_o: Some(branch_node_0_plum_head_seal.clone()),
                height: branch_node_0
                    .height
                    .checked_add(1)
                    .expect("height overflow"),
                metadata: metadata_1_plum_head_seal.clone(),
                content_o: Some(content_1_plum_head_seal.clone()),
                posi_diff_o: None,
                nega_diff_o: None,
            };
            let branch_node_1_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &branch_node_1,
                    Some(&ContentFormat::json()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let branch_node_1_plum_head_seal = datahost
                .store_plum(&branch_node_1_plum, None)
                .block_on()
                .expect("pass");

            let branch_node_2 = BranchNode {
                ancestor_o: Some(branch_node_1_plum_head_seal.clone()),
                height: branch_node_1
                    .height
                    .checked_add(1)
                    .expect("height overflow"),
                metadata: metadata_2_plum_head_seal.clone(),
                content_o: Some(content_2_plum_head_seal.clone()),
                posi_diff_o: None,
                nega_diff_o: None,
            };
            let branch_node_2_plum = PlumBuilder::new()
                .with_plum_relations_and_plum_body_content_from(
                    &branch_node_2,
                    Some(&ContentFormat::json()),
                    ContentEncoding::none(),
                )
                .expect("pass")
                .build()
                .expect("pass");
            let branch_node_2_plum_head_seal = datahost
                .store_plum(&branch_node_2_plum, None)
                .block_on()
                .expect("pass");
        }

        let view_stack_v = Vec::new();
        App {
            plum_table_view: PlumTableView::new(),
            view_stack_v,
            debug: false,
            datahost,
        }
    }

    fn title(&self) -> String {
        "Indoor Data Plumbing".to_string()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::BackPressed => {
                self.view_stack_v.pop();
            }
            Message::ForwardPressed(plum_head_seal) => {
                use pollster::FutureExt;
                self.view_stack_v.push(
                    PlumView::new(plum_head_seal, &self.datahost)
                        .block_on()
                        .unwrap(),
                );
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut controls = iced::widget::column![];
        controls = controls.push(
            iced::widget::button::Button::new(iced::widget::Text::new("Back"))
                .on_press(Message::BackPressed),
        );
        controls = controls.push(iced::widget::horizontal_rule(1));

        if let Some(focused_view) = self.view_stack_v.last() {
            controls = controls.push(focused_view.view(&self.datahost, self.debug))
        } else {
            controls = controls.push(self.plum_table_view.view(&self.datahost, self.debug));
        }

        controls.into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
