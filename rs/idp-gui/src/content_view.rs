use crate::{Message, PlumPreview};
use iced::Element;
use idp_core::{BranchNode, Datahost};
use idp_proto::{decode_and_deserialize_from_content, Content};
use idp_sig::{OwnedData, PlumSig};

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
            s if s == OwnedData::content_class_str() => {
                let owned_data: OwnedData =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                let mut col = iced::widget::column![];
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text(format!("Owner: {}", owned_data.owner)));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Data: "));
                    row =
                        PlumPreview.view(None, &owned_data.data, None, Some(row), datahost, debug);
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Previous: "));
                    if let Some(previous_owned_data) = owned_data.previous_owned_data_o.as_ref() {
                        row = PlumPreview.view(
                            None,
                            previous_owned_data,
                            None,
                            Some(row),
                            datahost,
                            debug,
                        );
                    } else {
                        row = row.push(iced::widget::text("None"));
                    }
                    col = col.push(row);
                }

                col.into()
            }
            s if s == PlumSig::content_class_str() => {
                let plum_sig: PlumSig =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                let mut col = iced::widget::column![];
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text(format!(
                        "Nonce: {}",
                        plum_sig.content.nonce
                    )));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Plum: "));
                    row = PlumPreview.view(
                        None,
                        &plum_sig.content.plum,
                        None,
                        Some(row),
                        datahost,
                        debug,
                    );
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text("Previous: "));
                    if let Some(previous_plum_sig) = plum_sig.content.previous_plum_sig_o.as_ref() {
                        row = PlumPreview.view(
                            None,
                            previous_plum_sig,
                            None,
                            Some(row),
                            datahost,
                            debug,
                        );
                    } else {
                        row = row.push(iced::widget::text("None"));
                    }
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text(format!(
                        "Signature: {}",
                        plum_sig.signature
                    )));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![];
                    use pollster::FutureExt;
                    // TODO: Should verify the whole chain
                    let is_verified = plum_sig.verify_and_extract_signer().block_on().is_ok();
                    row = row.push(iced::widget::text(format!(
                        "Signature is {}",
                        if is_verified { "Valid" } else { "INVALID" }
                    )));
                    col = col.push(row);
                }

                col.into()
            }
            _ => iced::widget::text("<no preview available>").into(),
        }
    }
}
