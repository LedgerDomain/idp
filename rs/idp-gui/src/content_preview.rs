use crate::Message;
use iced::Element;
use idp_core::{BranchNode, Datahost};
use idp_proto::{decode_and_deserialize_from_content, Content};

// This is meant to be shown within a single line.
pub struct ContentPreview;

impl ContentPreview {
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
                use pollster::FutureExt;
                let branch_node_metadata = datahost
                    .load_plum(&branch_node.metadata, None)
                    .block_on()
                    .expect("todo: handle error");
                self.view(
                    &branch_node_metadata.plum_body.plum_body_content,
                    datahost,
                    debug,
                )
            }
            _ => iced::widget::text("<no preview available>").into(),
        }
    }
}
