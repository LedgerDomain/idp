use crate::{Message, PlumPreview};
use iced::Element;
use idp_core::Datahost;

pub struct PlumTableView {}

impl PlumTableView {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, _datahost: &mut Datahost, _message: Message, _debug: &mut bool) {
        unimplemented!("todo");
    }

    pub fn view(&self, datahost: &Datahost, debug: bool) -> Element<Message> {
        let mut col = iced::widget::column![iced::widget::text("Plum Table")];
        col = col.push(iced::widget::horizontal_rule(1));

        // TODO: Render a real table

        use pollster::FutureExt;
        for (row_inserted_at, plum_head_seal, _plum_head) in
            datahost.select_plum_heads(None).block_on().unwrap()
        {
            col = col.push(PlumPreview.view(
                Some(row_inserted_at),
                &plum_head_seal,
                None,
                None,
                datahost,
                debug,
            ));
        }

        iced::widget::scrollable(col).into()
    }
}
