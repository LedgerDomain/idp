use iced_native::Alignment;
use idp_core::Datahost;
use idp_proto::{Plum, PlumHeadSeal, UnixNanoseconds};

use crate::{ContentPreview, Grid, Message, PlumHeadSealView};

pub struct PlumPreview;

impl PlumPreview {
    pub fn grid_view_column_count() -> usize {
        PlumHeadSealView::grid_view_column_count() + 5
    }
    pub fn grid_view_push_column_headers<'a>(
        mut grid: Grid<'a, Message, iced::Renderer>,
        debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        grid = PlumHeadSealView::grid_view_push_column_headers(grid, debug);
        grid.push(iced::widget::text("Stored At"))
            .push(iced::widget::text("Class"))
            .push(iced::widget::text("Format"))
            .push(iced::widget::text("Encoding"))
            .push(iced::widget::text("Content Preview"))
    }
    pub fn grid_view_push_row<'a>(
        &'a self,
        mut grid: Grid<'a, Message, iced::Renderer>,
        row_inserted_at_o: Option<UnixNanoseconds>,
        plum_head_seal: &PlumHeadSeal,
        plum_o: Option<&Plum>,
        datahost: &Datahost,
        debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        grid = PlumHeadSealView.grid_view_push_row(grid, plum_head_seal, debug);

        if let Some(row_inserted_at) = row_inserted_at_o {
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_inserted_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_inserted_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            grid = grid.push(iced::widget::text(row_inserted_at_local.to_string()));
        } else {
            grid = grid.push(iced::widget::text(""));
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

        grid = grid.push(iced::widget::text(
            plum.plum_body
                .plum_body_content
                .content_metadata
                .content_class
                .value
                .clone(),
        ));

        grid = grid.push(iced::widget::text(
            plum.plum_body
                .plum_body_content
                .content_metadata
                .content_format
                .value
                .clone(),
        ));

        grid = grid.push(iced::widget::text(
            plum.plum_body
                .plum_body_content
                .content_metadata
                .content_encoding
                .value
                .clone(),
        ));

        ContentPreview.grid_view_push_row(grid, &plum.plum_body.plum_body_content, datahost, debug)
    }
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
            iced::widget::row![].align_items(Alignment::Center)
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
        row = row.push(iced::widget::text("Content Preview: "));
        row = row.push(ContentPreview.view(&plum.plum_body.plum_body_content, datahost, debug));

        row
    }
}
