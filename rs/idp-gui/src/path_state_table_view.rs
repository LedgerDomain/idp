use crate::{Grid, Message, PathStatePreview};
use iced::Element;
use idp_core::Datahost;

pub struct PathStateTableView {}

impl PathStateTableView {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, _datahost: &mut Datahost, _message: Message, _debug: &mut bool) {
        unimplemented!("todo");
    }

    pub fn grid_view(&self, datahost: &Datahost, debug: bool) -> Element<Message> {
        let mut col = iced::widget::column![iced::widget::text("PathState Table")];
        col = col.push(iced::widget::horizontal_rule(1));

        let column_count = PathStatePreview::grid_view_column_count();
        let mut grid = PathStatePreview::grid_view_push_column_headers(
            Grid::with_columns(column_count)
                .horizontal_spacing(10)
                .vertical_spacing(10),
            debug,
        );

        use pollster::FutureExt;
        for (row_inserted_at, row_updated_at, path_state) in
            datahost.select_path_states(None).block_on().unwrap()
        {
            grid = PathStatePreview.grid_view_push_row(
                grid,
                Some(row_inserted_at),
                Some(row_updated_at),
                &path_state,
                datahost,
                debug,
            );
        }
        col = col.push(grid);

        iced::widget::scrollable(col).into()
    }
    pub fn view(&self, datahost: &Datahost, debug: bool) -> Element<Message> {
        let mut col = iced::widget::column![iced::widget::text("PathState Table")];
        col = col.push(iced::widget::horizontal_rule(1));

        // TODO: Render a real table

        use pollster::FutureExt;
        for (row_inserted_at, row_updated_at, path_state) in
            datahost.select_path_states(None).block_on().unwrap()
        {
            col = col.push(PathStatePreview.view(
                Some(row_inserted_at),
                Some(row_updated_at),
                &path_state,
                None,
                datahost,
                debug,
            ));
        }

        iced::widget::scrollable(col).into()
    }
}
