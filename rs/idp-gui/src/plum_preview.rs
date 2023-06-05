use idp_core::Datahost;
use idp_proto::{Plum, PlumHeadSeal, UnixNanoseconds};

use crate::{ContentPreview, Message, PlumHeadSealView};

pub struct PlumPreview;

impl PlumPreview {
    pub fn view<'a>(
        &'a self,
        row_inserted_at_o: Option<UnixNanoseconds>,
        plum_head_seal: &PlumHeadSeal,
        plum_o: Option<&Plum>,
        continue_row_o: Option<iced::widget::Row<'a, Message>>,
        datahost: &Datahost,
        debug: bool,
    ) -> iced::widget::Row<'a, Message> {
        let mut row = if let Some(row) = continue_row_o {
            row
        } else {
            iced::widget::row![]
        };

        row = row.push(PlumHeadSealView.view(&plum_head_seal, debug));

        if let Some(row_inserted_at) = row_inserted_at_o {
            row = row.push(iced::widget::horizontal_space(10));
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_inserted_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_inserted_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            row = row.push(iced::widget::text(format!(
                // "Stored: {}",
                "{}",
                row_inserted_at_local
            )));
        }

        let loaded_plum_o = if plum_o.is_none() {
            use pollster::FutureExt;
            Some(
                datahost
                    .load_plum(&plum_head_seal, None)
                    .block_on()
                    .unwrap(),
            )
        } else {
            None
        };
        let plum = if let Some(plum) = plum_o {
            plum
        } else {
            loaded_plum_o.as_ref().unwrap()
        };

        row = row.push(iced::widget::horizontal_space(10));
        row = row.push(iced::widget::text(format!(
            // "Class: {}",
            "{}",
            plum.plum_body
                .plum_body_content
                .content_metadata
                .content_class
                .value,
        )));

        row = row.push(iced::widget::horizontal_space(10));
        row = row.push(iced::widget::text(format!(
            // "Format: {}",
            "{}",
            plum.plum_body
                .plum_body_content
                .content_metadata
                .content_format
                .value,
        )));

        // row = row.push(iced::widget::horizontal_space(10));
        // row = row.push(iced::widget::text(format!(
        //     "Encoding: {}",
        //     plum.plum_body
        //         .plum_body_content
        //         .content_metadata
        //         .content_encoding
        //         .value,
        // )));

        row = row.push(iced::widget::horizontal_space(10));
        row = row.push(iced::widget::text("Content: "));
        row = row.push(ContentPreview.view(&plum.plum_body.plum_body_content, datahost, debug));

        row
    }
}
