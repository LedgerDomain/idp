use crate::{Grid, Message};
use iced::Element;
use iced_native::{alignment::Vertical, Alignment};
use idp_core::{BranchNode, Datahost};
use idp_proto::{decode_and_deserialize_from_content, Content};
use idp_sig::{OwnedData, PlumSig};

// This is meant to be shown within a single line.
pub struct ContentPreview;

impl ContentPreview {
    pub fn update(&mut self, _message: Message, _debug: &mut bool) {
        unimplemented!("todo");
    }

    pub fn grid_view_push_row<'a>(
        &'a self,
        grid: Grid<'a, Message, iced::Renderer>,
        content: &Content,
        datahost: &Datahost,
        debug: bool,
    ) -> Grid<'a, Message, iced::Renderer> {
        use idp_proto::ContentClassifiable;
        match content.content_metadata.content_class.as_str() {
            "text/plain" | "application/json" => grid.push(
                iced::widget::text(format!(
                    "{}",
                    match std::str::from_utf8(content.content_byte_v.as_slice()) {
                        Ok(s) => s,
                        Err(_) => "<invalid UTF-8>",
                    }
                ))
                .vertical_alignment(Vertical::Center),
            ),
            s if s == BranchNode::content_class_str() => {
                let branch_node: BranchNode =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                use pollster::FutureExt;
                let branch_node_metadata = datahost
                    .load_plum(&branch_node.metadata, None)
                    .block_on()
                    .expect("todo: handle error");
                self.grid_view_push_row(
                    grid,
                    &branch_node_metadata.plum_body.plum_body_content,
                    datahost,
                    debug,
                )
            }
            s if s == OwnedData::content_class_str() => {
                let owned_data: OwnedData =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                grid.push(
                    iced::widget::text(format!("Owner: {}", owned_data.owner))
                        .vertical_alignment(Vertical::Center),
                )
            }
            s if s == PlumSig::content_class_str() => {
                let plum_sig: PlumSig =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                use pollster::FutureExt;
                // TODO: Should verify the whole chain
                if let Ok(signer_did_fragment_url) = plum_sig.verify_and_extract_signer().block_on()
                {
                    grid.push(
                        iced::widget::row![
                            iced_aw::native::Badge::new(iced::widget::text("Valid"))
                                .style(iced_aw::style::BadgeStyles::Success),
                            iced::widget::text(format!(
                                "Signature by: {}",
                                signer_did_fragment_url.did
                            )),
                        ]
                        .align_items(Alignment::Center),
                    )
                } else {
                    let signer_did = plum_sig
                        .signature
                        .extract_signer_did_fragment_url()
                        .unwrap()
                        .did;
                    grid.push(
                        iced::widget::row![
                            iced_aw::native::Badge::new(iced::widget::text("INVALID"))
                                .style(iced_aw::style::BadgeStyles::Danger),
                            iced::widget::text(format!("Signature by: {}", signer_did)),
                        ]
                        .align_items(Alignment::Center),
                    )
                }
            }
            _ => grid.push(
                iced::widget::text("<no preview available>").vertical_alignment(Vertical::Center),
            ),
        }
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
            s if s == OwnedData::content_class_str() => {
                let owned_data: OwnedData =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                iced::widget::text(format!("Owner: {}", owned_data.owner)).into()
            }
            s if s == PlumSig::content_class_str() => {
                let plum_sig: PlumSig =
                    decode_and_deserialize_from_content(content).expect("todo: handle error");
                use pollster::FutureExt;
                // TODO: Should verify the whole chain
                if let Ok(signer_did_fragment_url) = plum_sig.verify_and_extract_signer().block_on()
                {
                    iced::widget::row![
                        iced_aw::native::Badge::new(iced::widget::text("Valid"))
                            .style(iced_aw::style::BadgeStyles::Success),
                        iced::widget::text(format!(
                            "Signature by: {}",
                            signer_did_fragment_url.did
                        )),
                    ]
                    .align_items(Alignment::Center)
                    .into()
                } else {
                    let signer_did = plum_sig
                        .signature
                        .extract_signer_did_fragment_url()
                        .unwrap()
                        .did;
                    iced::widget::row![
                        iced_aw::native::Badge::new(iced::widget::text("INVALID"))
                            .style(iced_aw::style::BadgeStyles::Danger),
                        iced::widget::text(format!("Signature by: {}", signer_did)),
                    ]
                    .align_items(Alignment::Center)
                    .into()
                }
            }
            _ => iced::widget::text("<no preview available>").into(),
        }
    }
}
