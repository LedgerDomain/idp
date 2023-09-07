use crate::{Message, PlumPreview};
use iced::Element;
use iced_native::Alignment;
use idp_core::{BranchNode, Datahost, DirNode};
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
                match std::str::from_utf8(content.content_byte_v.as_slice()) {
                    Ok(s) => s,
                    Err(_) => "<invalid UTF-8>",
                }
            ))
            .into(),
            s if s == BranchNode::content_class_str() => {
                let branch_node: BranchNode =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                let mut col = iced::widget::column![];
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text("Ancestor: "));
                    if let Some(ancestor) = branch_node.ancestor_o.as_ref() {
                        row = PlumPreview.view(None, ancestor, None, Some(row), datahost, debug);
                    } else {
                        row = row.push(iced::widget::text("None"));
                    }
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text("Metadata: "));
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
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text("Content: "));
                    if let Some(content) = branch_node.content_o.as_ref() {
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
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text(format!("Owner: {}", owned_data.owner)));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text("Data: "));
                    row =
                        PlumPreview.view(None, &owned_data.data, None, Some(row), datahost, debug);
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
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
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text(format!(
                        "Nonce: {}",
                        plum_sig.content.nonce
                    )));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
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
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
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
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text(format!(
                        "Signature: {}",
                        plum_sig.signature
                    )));
                    col = col.push(row);
                }
                {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
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
            s if s == DirNode::content_class_str() => {
                let dir_node: DirNode =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                let mut col = iced::widget::column![];
                for (entry_name, target_plum_head_seal) in dir_node.entry_m.iter() {
                    let mut row = iced::widget::row![].align_items(Alignment::Center);
                    row = row.push(iced::widget::text(format!("{}: ", entry_name)));
                    row = PlumPreview.view(
                        None,
                        target_plum_head_seal,
                        None,
                        Some(row),
                        datahost,
                        debug,
                    );
                    col = col.push(row);
                }
                col.into()
            }
            _ => iced::widget::text("<no preview available>").into(),
        }
    }
}
