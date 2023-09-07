use iced::Element;
use idp_proto::PlumHeadSeal;

use crate::{Grid, Message};

pub struct PlumHeadSealView;

impl PlumHeadSealView {
    pub fn grid_view_column_count() -> usize {
        1
    }
    pub fn grid_view_push_column_headers<'a>(
        grid: Grid<'a, Message, iced::Renderer>,
        _debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        grid.push(iced::widget::text("Plum Head Seal"))
    }
    pub fn grid_view_push_row<'a>(
        &'a self,
        grid: Grid<'a, Message, iced::Renderer>,
        plum_head_seal: &PlumHeadSeal,
        _debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        let mut plum_head_seal_string = plum_head_seal.to_string();
        plum_head_seal_string.truncate(8);
        grid.push(
            iced::widget::button::Button::new(iced::widget::text(plum_head_seal_string))
                .on_press(Message::ForwardPressed(plum_head_seal.clone())),
        )
    }
    pub fn view(&self, plum_head_seal: &PlumHeadSeal, _debug: bool) -> Element<Message> {
        let mut plum_head_seal_string = plum_head_seal.to_string();
        plum_head_seal_string.truncate(8);
        iced::widget::button::Button::new(iced::widget::text(plum_head_seal_string))
            .on_press(Message::ForwardPressed(plum_head_seal.clone()))
            .into()

        // let mut row = iced::widget::row![].align_items(Alignment::Center);

        // let mut plum_head_seal_string = plum_head_seal.to_string();
        // plum_head_seal_string.truncate(8);

        // row = row.push(
        //     iced::widget::button::Button::new(iced::widget::text(plum_head_seal_string))
        //         .on_press(Message::ForwardPressed(plum_head_seal.clone())), // .into(),
        // );
        // // Clipboard icon to copy PlumHeadSeal to clipboard
        // row = row.push(
        //     iced::widget::button::Button::new(iced::widget::text("📋"))
        //         .on_press(Message::CopyToClipboard(plum_head_seal.to_string())), // .into(),
        // );

        // row.into()
    }
}
