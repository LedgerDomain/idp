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
            let plum = datahost
                .load_plum(&plum_head_seal, None)
                .block_on()
                .unwrap();

            // // let mut row: iced::widget::Row<'_, Message, _> = iced::widget::row![];
            // let mut row = iced::widget::row![];

            // row = row.push(PlumHeadSealView.view(&plum_head_seal, debug));

            // row = row.push(iced::widget::horizontal_space(10));
            // // Hacky way to round to the nearest second to shorted the timestamp.
            // let row_inserted_at_local = chrono::DateTime::<chrono::Local>::from(
            //     UnixNanoseconds::from((row_inserted_at.value / 1_000_000_000) * 1_000_000_000),
            // )
            // .naive_local();

            // row = row.push(iced::widget::text(format!(
            //     // "Stored: {}",
            //     "{}",
            //     row_inserted_at_local
            // )));

            // row = row.push(iced::widget::horizontal_space(10));
            // row = row.push(iced::widget::text(format!(
            //     // "Class: {}",
            //     "{}",
            //     plum.plum_body
            //         .plum_body_content
            //         .content_metadata
            //         .content_class
            //         .value,
            // )));

            // row = row.push(iced::widget::horizontal_space(10));
            // row = row.push(iced::widget::text(format!(
            //     // "Format: {}",
            //     "{}",
            //     plum.plum_body
            //         .plum_body_content
            //         .content_metadata
            //         .content_format
            //         .value,
            // )));

            // // row = row.push(iced::widget::horizontal_space(10));
            // // row = row.push(iced::widget::text(format!(
            // //     "Encoding: {}",
            // //     plum.plum_body
            // //         .plum_body_content
            // //         .content_metadata
            // //         .content_encoding
            // //         .value,
            // // )));

            // row = row.push(iced::widget::text("Content: "));
            // row = row.push(iced::widget::horizontal_space(10));
            // row = row.push(ContentPreview.view(&plum.plum_body.plum_body_content, datahost, debug));

            // col = col.push(row);
            col = col.push(PlumPreview.view(
                Some(row_inserted_at),
                &plum_head_seal,
                Some(&plum),
                None,
                datahost,
                debug,
            ));
        }

        iced::widget::scrollable(col).into()
        // controls.into()
    }
}
