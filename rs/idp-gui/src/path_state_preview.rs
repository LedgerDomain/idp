use iced_native::{alignment::Vertical, Alignment};
use idp_core::Datahost;
use idp_proto::{PathState, UnixNanoseconds};

use crate::{Grid, Message, PlumPreview};

pub struct PathStatePreview;

impl PathStatePreview {
    pub fn grid_view_column_count() -> usize {
        3 + PlumPreview::grid_view_column_count()
    }
    pub fn grid_view_push_column_headers<'a>(
        mut grid: Grid<'a, Message, iced::Renderer>,
        debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        grid = grid
            .push(iced::widget::text("Path").vertical_alignment(Vertical::Center))
            .push(iced::widget::text("Created At").vertical_alignment(Vertical::Center))
            .push(iced::widget::text("Updated At").vertical_alignment(Vertical::Center));
        PlumPreview::grid_view_push_column_headers(grid, debug)
    }
    pub fn grid_view_push_row<'a>(
        &'a self,
        mut grid: Grid<'a, Message, iced::Renderer>,
        row_inserted_at_o: Option<UnixNanoseconds>,
        row_updated_at_o: Option<UnixNanoseconds>,
        path_state: &PathState,
        datahost: &Datahost,
        debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        // TODO: Use as_str; requires keeping the PathState in memory.
        grid = grid
            .push(iced::widget::text(path_state.path.clone()).vertical_alignment(Vertical::Center));

        if let Some(row_inserted_at) = row_inserted_at_o {
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_inserted_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_inserted_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            grid = grid.push(
                iced::widget::text(row_inserted_at_local.to_string())
                    .vertical_alignment(Vertical::Center),
            );
        } else {
            grid = grid.push(iced::widget::text("").vertical_alignment(Vertical::Center));
        }

        if let Some(row_updated_at) = row_updated_at_o {
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_updated_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_updated_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            grid = grid.push(
                iced::widget::text(row_updated_at_local.to_string())
                    .vertical_alignment(Vertical::Center),
            );
        } else {
            grid = grid.push(iced::widget::text("").vertical_alignment(Vertical::Center));
        }

        PlumPreview.grid_view_push_row(
            grid,
            None,
            &path_state.current_state_plum_head_seal,
            None,
            datahost,
            debug,
        )
    }
    pub fn view<'a>(
        &'a self,
        row_inserted_at_o: Option<UnixNanoseconds>,
        row_updated_at_o: Option<UnixNanoseconds>,
        path_state: &PathState,
        continue_row_o: Option<iced::widget::Row<'a, Message>>,
        datahost: &Datahost,
        debug: bool,
    ) -> iced::widget::Row<'a, Message> {
        let mut row = if let Some(row) = continue_row_o {
            row
        } else {
            iced::widget::row![].align_items(Alignment::Center)
        };

        row = row.push(iced::widget::text(format!("Path: {}", path_state.path,)));

        if let Some(row_inserted_at) = row_inserted_at_o {
            row = row.push(iced::widget::horizontal_space(10));
            row = row.push(iced::widget::horizontal_space(10));
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_inserted_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_inserted_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            row = row.push(iced::widget::text(format!(
                "Created At: {}",
                row_inserted_at_local
            )));
        }

        if let Some(row_updated_at) = row_updated_at_o {
            row = row.push(iced::widget::horizontal_space(10));
            row = row.push(iced::widget::horizontal_space(10));
            // Hacky way to round to the nearest second to shorted the timestamp.
            let row_updated_at_local = chrono::DateTime::<chrono::Local>::from(
                UnixNanoseconds::from((row_updated_at.value / 1_000_000_000) * 1_000_000_000),
            )
            .naive_local();

            row = row.push(iced::widget::text(format!(
                "Updated At: {}",
                row_updated_at_local
            )));
        }

        row = row.push(iced::widget::horizontal_space(10));
        row = PlumPreview.view(
            None,
            &path_state.current_state_plum_head_seal,
            None,
            Some(row),
            datahost,
            debug,
        );

        row
    }
}
