use crate::{Message, PlumPreview};
use iced::Element;
use idp_core::{BranchNode, Datahost};
use idp_proto::{decode_and_deserialize_from_content, Content};

// This is meant to show the whole content.
pub struct ContentView;

impl ContentView {
    pub fn update(&mut self, _message: Message, _debug: &mut bool) {
        unimplemented!("todo");
    }

    pub fn view(&self, content: &Content, datahost: &Datahost, debug: bool) -> Element<Message> {
        use idp_proto::ContentClassifiable;
        match content.content_metadata.content_class.as_str() {
            "text/plain" | "application/json" => iced::widget::text(format!(
                "{}",
                String::from_utf8_lossy(content.content_byte_v.as_slice())
            ))
            .into(),
            s if s == BranchNode::content_class_str() => {
                let branch_node: BranchNode =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                let mut col = iced::widget::column![];
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Ancestor: "));
                    if let Some(ancestor) = branch_node.ancestor_o.as_ref() {
                        // row = row.push(PlumHeadSealView.view(ancestor, debug));
                        row = PlumPreview.view(None, ancestor, None, Some(row), datahost, debug);
                    } else {
                        row = row.push(iced::widget::text("None"));
                    }
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Metadata: "));
                    // row = row.push(PlumHeadSealView.view(&branch_node.metadata, debug));
                    row = PlumPreview.view(
                        None,
                        &branch_node.metadata,
                        None,
                        Some(row),
                        datahost,
                        debug,
                    );
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Content: "));
                    if let Some(content) = branch_node.content_o.as_ref() {
                        // row = row.push(PlumHeadSealView.view(content, debug));
                        row = PlumPreview.view(None, content, None, Some(row), datahost, debug);
                    } else {
                        row = row.push(iced::widget::text("None"));
                    }
                    col = col.push(row);
                }

                col.into()
            }
            _ => iced::widget::text("<no preview available>").into(),
        }
    }
}
