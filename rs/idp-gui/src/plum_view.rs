use crate::{ContentView, Message, PlumPreview};
use iced::Element;
use idp_core::Datahost;
use idp_proto::{Plum, PlumHeadSeal, PlumRelationFlags, UnixNanoseconds};

pub struct PlumView {
    plum_head_seal: PlumHeadSeal,
    // TODO: Other view state
    plum_o: Option<Plum>,
}

impl PlumView {
    pub async fn new(plum_head_seal: PlumHeadSeal, datahost: &Datahost) -> anyhow::Result<Self> {
        let plum_o = datahost.load_option_plum(&plum_head_seal, None).await?;
        Ok(Self {
            plum_head_seal,
            plum_o,
        })
    }
    pub fn update(&mut self, _message: Message, _debug: &mut bool) {
        // self.steps[self.current].update(message, debug);
        unimplemented!("todo");
    }

    pub fn view(&self, datahost: &Datahost, debug: bool) -> Element<Message> {
        let mut col =
            iced::widget::column![iced::widget::text(format!("Plum {}", self.plum_head_seal))];
        col = col.push(iced::widget::horizontal_rule(1));
        if let Some(plum) = self.plum_o.as_ref() {
            // Show the PlumHead
            {
                let plum_head = &plum.plum_head;
                col = col.push("Head");
                col = col.push(iced::widget::horizontal_rule(1));
                if let Some(plum_head_nonce) = plum_head.plum_head_nonce_o.as_ref() {
                    col = col.push(iced::widget::text(format!(
                        "Head Nonce: {}",
                        plum_head_nonce
                    )));
                } else {
                    col = col.push(iced::widget::text("Head Nonce: None"));
                }
                col = col.push(iced::widget::text(format!(
                    "Metadata Seal: {}",
                    plum_head.plum_metadata_seal,
                )));
                col = col.push(iced::widget::text(format!(
                    "Relations Seal: {}",
                    plum_head.plum_relations_seal,
                )));
                col = col.push(iced::widget::text(format!(
                    "Body Seal: {}",
                    plum_head.plum_body_seal,
                )));
                col = col.push(iced::widget::horizontal_rule(1));
            }
            // Show the PlumMetadata
            {
                let plum_metadata = &plum.plum_metadata;
                col = col.push("Metadata");
                col = col.push(iced::widget::horizontal_rule(1));
                if let Some(plum_metadata_nonce) = plum_metadata.plum_metadata_nonce_o.as_ref() {
                    col = col.push(iced::widget::text(format!(
                        "Metadata Nonce: {}",
                        plum_metadata_nonce
                    )));
                } else {
                    col = col.push(iced::widget::text("Metadata Nonce: None"));
                }
                if let Some(plum_created_at) = plum_metadata.plum_created_at_o.as_ref() {
                    // Hacky way to round to the nearest second to shorted the timestamp.
                    let plum_created_at_local =
                        chrono::DateTime::<chrono::Local>::from(UnixNanoseconds::from(
                            (plum_created_at.value / 1_000_000_000) * 1_000_000_000,
                        ));
                    // controls = controls.push(iced::widget::text(format!(
                    //     "Plum Created At: {} ({} ns since Unix epoch)",
                    //     plum_created_at_utc, plum_created_at.value
                    // )));
                    col = col.push(iced::widget::text(format!(
                        "Plum Created At: {}",
                        plum_created_at_local
                    )));
                } else {
                    col = col.push(iced::widget::text("Plum Created At: None"));
                }
                if let Some(plum_body_content_metadata) =
                    plum_metadata.plum_body_content_metadata_o.as_ref()
                {
                    col = col.push(iced::widget::text(format!(
                        "Body Content Length: {}",
                        plum_body_content_metadata.content_length
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Class: {}",
                        plum_body_content_metadata.content_class.value
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Format: {}",
                        plum_body_content_metadata.content_format.value
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Encoding: {}",
                        plum_body_content_metadata.content_encoding.value
                    )));
                } else {
                    col = col.push(iced::widget::text("Body Content Metadata: Not Included"));
                }
                // TODO: Additional content
                col = col.push(iced::widget::horizontal_rule(1));
            }
            // Show the PlumRelations
            {
                let plum_relations = &plum.plum_relations;
                col = col.push("Relations");
                col = col.push(iced::widget::horizontal_rule(1));
                if let Some(plum_relations_nonce) = plum_relations.plum_relations_nonce_o.as_ref() {
                    col = col.push(iced::widget::text(format!(
                        "Relations Nonce: {}",
                        plum_relations_nonce
                    )));
                } else {
                    col = col.push(iced::widget::text("Relations Nonce: None"));
                }
                col = col.push(iced::widget::text(format!(
                    "Relations Entries ({} In Total):",
                    plum_relations.plum_relation_flags_mapping_v.len()
                )));
                for plum_relation_flags_mapping in
                    plum_relations.plum_relation_flags_mapping_v.iter()
                {
                    let plum_relations_flags = PlumRelationFlags::try_from(
                        plum_relation_flags_mapping.plum_relation_flags_raw,
                    )
                    .unwrap();

                    let mut row = iced::widget::row![];
                    row = row.push(iced::widget::text(format!("{:?}", plum_relations_flags)));
                    let row = PlumPreview.view(
                        None,
                        &plum_relation_flags_mapping.target_plum_head_seal,
                        None,
                        Some(row),
                        datahost,
                        debug,
                    );
                    col = col.push(row);
                }
                col = col.push(iced::widget::horizontal_rule(1));
            }
            // Show the PlumBody
            {
                let plum_body = &plum.plum_body;
                col = col.push("Body");
                col = col.push(iced::widget::horizontal_rule(1));
                if let Some(plum_body_nonce) = plum_body.plum_body_nonce_o.as_ref() {
                    col = col.push(iced::widget::text(format!(
                        "Body Nonce: {}",
                        plum_body_nonce
                    )));
                } else {
                    col = col.push(iced::widget::text("Body Nonce: None"));
                }
                {
                    let plum_body_content_metadata = &plum_body.plum_body_content.content_metadata;
                    col = col.push(iced::widget::text(format!(
                        "Body Content Length: {}",
                        plum_body_content_metadata.content_length
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Class: {}",
                        plum_body_content_metadata.content_class.value
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Format: {}",
                        plum_body_content_metadata.content_format.value
                    )));
                    col = col.push(iced::widget::text(format!(
                        "Body Content Encoding: {}",
                        plum_body_content_metadata.content_encoding.value
                    )));
                }
                col = col.push(iced::widget::text("Body Content Follows"));
                col = col.push(iced::widget::horizontal_rule(1));
                col = col.push(ContentView.view(&plum_body.plum_body_content, datahost, debug));
            }
        } else {
            col = col.push("Plum not present in this Datahost");
        }
        // TODO: More stuff

        iced::widget::scrollable(col).into()
    }
}
