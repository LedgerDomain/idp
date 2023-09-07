use crate::{Message, PathStateTableView, PlumTableView, PlumView};
use iced::{Application, Command, Element, Subscription, Theme};
use idp_core::{BranchNode, DirNode};
use idp_proto::{
    ContentEncoding, ContentFormat, Nonce, Path, PathState, PlumBuilder, UnixNanoseconds,
};
use idp_sig::{OwnedData, PlumSig, PlumSigContent};

pub struct App {
    path_state_table_view: PathStateTableView,
    #[allow(unused)]
    plum_table_view: PlumTableView,
    // NOTE: Later this will be a generic View object
    view_stack_v: Vec<PlumView>,
    debug: bool,
    datahost: idp_core::Datahost,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn title(&self) -> String {
        "Indoor Data Plumbing".to_string()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            Message::BackPressed => {
                self.view_stack_v.pop();
            }
            // Message::CopyToClipboard(string) => {}
            Message::ForwardPressed(plum_head_seal) => {
                use pollster::FutureExt;
                self.view_stack_v.push(
                    PlumView::new(plum_head_seal, &self.datahost)
                        .block_on()
                        .unwrap(),
                );
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut controls = iced::widget::column![];
        controls = controls.push(
            iced::widget::button::Button::new(iced::widget::Text::new("Back"))
                .on_press(Message::BackPressed),
        );
        controls = controls.push(iced::widget::horizontal_rule(1));

        if let Some(focused_view) = self.view_stack_v.last() {
            controls = controls.push(focused_view.view(&self.datahost, self.debug))
        } else {
            // controls = controls.push(self.plum_table_view.view(&self.datahost, self.debug));
            // controls = controls.push(self.path_state_table_view.view(&self.datahost, self.debug));
            controls = controls.push(
                self.path_state_table_view
                    .grid_view(&self.datahost, self.debug),
            );
        }

        controls.into()
    }
}

impl Default for App {
    fn default() -> Self {
        use pollster::FutureExt;
        // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
        let datahost_storage =
            idp_datahost_storage_sqlite::DatahostStorageSQLite::connect_and_run_migrations(
                "sqlite:idp-gui.db?mode=rwc",
            )
            .block_on()
            .expect("handle error");
        let mut datahost = idp_core::Datahost::open(datahost_storage);

        const CREATE_TEST_DATA: bool = false;

        if CREATE_TEST_DATA {
            // Make some BranchNode content
            {
                let content_1 = "splunges are super-duper cool".to_string();
                let content_2 = "HIPPOS are mega-cool".to_string();

                let content_1_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &content_1,
                        Some(&ContentFormat::charset_us_ascii()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let content_2_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &content_2,
                        Some(&ContentFormat::charset_us_ascii()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");

                let metadata_0_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &"Branch root".to_string(),
                        Some(&ContentFormat::charset_us_ascii()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let metadata_1_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &"Initial statement".to_string(),
                        Some(&ContentFormat::charset_us_ascii()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let metadata_2_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &"Revised statement authored by the HIPPO lobby".to_string(),
                        Some(&ContentFormat::charset_us_ascii()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");

                let content_1_plum_head_seal = datahost
                    .store_plum(&content_1_plum, None)
                    .block_on()
                    .expect("pass");
                let content_2_plum_head_seal = datahost
                    .store_plum(&content_2_plum, None)
                    .block_on()
                    .expect("pass");

                datahost
                    .insert_path_state(
                        &PathState {
                            path: Path::from("fancy-path".to_string()),
                            current_state_plum_head_seal: content_1_plum_head_seal.clone(),
                        },
                        None,
                    )
                    .block_on()
                    .expect("pass");

                let metadata_0_plum_head_seal = datahost
                    .store_plum(&metadata_0_plum, None)
                    .block_on()
                    .expect("pass");
                let metadata_1_plum_head_seal = datahost
                    .store_plum(&metadata_1_plum, None)
                    .block_on()
                    .expect("pass");
                let metadata_2_plum_head_seal = datahost
                    .store_plum(&metadata_2_plum, None)
                    .block_on()
                    .expect("pass");

                let branch_node_0 = BranchNode {
                    ancestor_o: None,
                    height: 0,
                    metadata: metadata_0_plum_head_seal.clone(),
                    content_o: None,
                    posi_diff_o: None,
                    nega_diff_o: None,
                };
                let branch_node_0_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &branch_node_0,
                        Some(&ContentFormat::json()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let branch_node_0_plum_head_seal = datahost
                    .store_plum(&branch_node_0_plum, None)
                    .block_on()
                    .expect("pass");

                let branch_node_1 = BranchNode {
                    ancestor_o: Some(branch_node_0_plum_head_seal.clone()),
                    height: branch_node_0
                        .height
                        .checked_add(1)
                        .expect("height overflow"),
                    metadata: metadata_1_plum_head_seal.clone(),
                    content_o: Some(content_1_plum_head_seal.clone()),
                    posi_diff_o: None,
                    nega_diff_o: None,
                };
                let branch_node_1_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &branch_node_1,
                        Some(&ContentFormat::json()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let branch_node_1_plum_head_seal = datahost
                    .store_plum(&branch_node_1_plum, None)
                    .block_on()
                    .expect("pass");

                let branch_node_2 = BranchNode {
                    ancestor_o: Some(branch_node_1_plum_head_seal.clone()),
                    height: branch_node_1
                        .height
                        .checked_add(1)
                        .expect("height overflow"),
                    metadata: metadata_2_plum_head_seal.clone(),
                    content_o: Some(content_2_plum_head_seal.clone()),
                    posi_diff_o: None,
                    nega_diff_o: None,
                };
                let branch_node_2_plum = PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &branch_node_2,
                        Some(&ContentFormat::json()),
                        ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass");
                let _branch_node_2_plum_head_seal = datahost
                    .store_plum(&branch_node_2_plum, None)
                    .block_on()
                    .expect("pass");
            }
            if true {
                // Generate 2 private keys for signing.  Each one represents a different owner.
                let signer_0_priv_jwk = idp_sig::KeyType::Secp256k1
                    .generate_priv_jwk()
                    .expect("pass");
                let signer_0_pub_jwk = signer_0_priv_jwk.to_public();
                let signer_0_did = idp_sig::did_key_from_jwk(&signer_0_pub_jwk)
                    .expect("pass")
                    .did;
                let signer_1_priv_jwk = idp_sig::KeyType::Secp256k1
                    .generate_priv_jwk()
                    .expect("pass");
                let signer_1_pub_jwk = signer_1_priv_jwk.to_public();
                let signer_1_did = idp_sig::did_key_from_jwk(&signer_1_pub_jwk)
                    .expect("pass")
                    .did;

                // Define the path name for the PlumSig
                let path = Path::from("important-signed-data".to_string());

                // Create a bunch of content Plum-s.  Note that instead of a loop, one could use
                // futures::future::try_join_all (see https://stackoverflow.com/questions/68344087/how-do-you-call-an-async-method-within-a-closure-like-within-map-in-rust),
                // and that would run all the async calls in parallel.
                let mut content_plum_head_seal_v = Vec::new();
                for content_str in [
                    "ostriches run all funky",
                    "donkeys run all regular",
                    "now *I* am the owner!",
                    "and *I* declare that humans rule!",
                ]
                .into_iter()
                {
                    let content_plum_head_seal = datahost
                        .store_plum(
                            &idp_proto::PlumBuilder::new()
                                .with_plum_relations_and_plum_body_content_from(
                                    &content_str.to_string(),
                                    None,
                                    idp_proto::ContentEncoding::none(),
                                )
                                .expect("pass")
                                .build()
                                .expect("pass"),
                            None,
                        )
                        .block_on()
                        .expect("pass");
                    content_plum_head_seal_v.push(content_plum_head_seal);
                }

                // Must use without_previous for the first PlumSig in a chain.
                let plum_sig_0_plum_head_seal =
                    idp_sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_without_previous(
                        &signer_0_priv_jwk,
                        content_plum_head_seal_v[0].clone(),
                        &mut datahost,
                        None,
                    )
                    .block_on()
                    .expect("pass");
                // Create the PathState.
                idp_sig::execute_path_state_plum_sig_create(
                    &mut datahost,
                    None,
                    path.clone(),
                    plum_sig_0_plum_head_seal.clone(),
                )
                .block_on()
                .expect("pass");
                // Verify it.
                assert_eq!(
                    datahost
                        .load_path_state(&path, None)
                        .block_on()
                        .expect("pass"),
                    idp_proto::PathState {
                        path: path.clone(),
                        current_state_plum_head_seal: plum_sig_0_plum_head_seal.clone()
                    }
                );
                idp_sig::PlumSig::verify_chain(&plum_sig_0_plum_head_seal, &mut datahost, None)
                    .block_on()
                    .expect("pass");

                let plum_sig_1_plum_head_seal =
                    idp_sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
                        plum_sig_0_plum_head_seal,
                        &signer_0_priv_jwk,
                        signer_0_did.clone(),
                        content_plum_head_seal_v[1].clone(),
                        &mut datahost,
                        None,
                    )
                    .block_on()
                    .expect("pass");
                // Update the PathState.
                idp_sig::execute_path_state_plum_sig_update(
                    &mut datahost,
                    None,
                    path.clone(),
                    plum_sig_1_plum_head_seal.clone(),
                )
                .block_on()
                .expect("pass");
                // Verify it.
                assert_eq!(
                    datahost
                        .load_path_state(&path, None)
                        .block_on()
                        .expect("pass"),
                    idp_proto::PathState {
                        path: path.clone(),
                        current_state_plum_head_seal: plum_sig_1_plum_head_seal.clone()
                    }
                );
                idp_sig::PlumSig::verify_chain(&plum_sig_1_plum_head_seal, &mut datahost, None)
                    .block_on()
                    .expect("pass");

                let plum_sig_2_plum_head_seal =
                    idp_sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
                        plum_sig_1_plum_head_seal,
                        &signer_0_priv_jwk,
                        // NOTE that the signer changed from signer_0_did to signer_1_did.
                        signer_1_did.clone(),
                        content_plum_head_seal_v[2].clone(),
                        &mut datahost,
                        None,
                    )
                    .block_on()
                    .expect("pass");
                // Update the PathState.
                idp_sig::execute_path_state_plum_sig_update(
                    &mut datahost,
                    None,
                    path.clone(),
                    plum_sig_2_plum_head_seal.clone(),
                )
                .block_on()
                .expect("pass");
                // Verify it
                assert_eq!(
                    datahost
                        .load_path_state(&path, None)
                        .block_on()
                        .expect("pass"),
                    idp_proto::PathState {
                        path: path.clone(),
                        current_state_plum_head_seal: plum_sig_2_plum_head_seal.clone()
                    }
                );
                idp_sig::PlumSig::verify_chain(&plum_sig_2_plum_head_seal, &mut datahost, None)
                    .block_on()
                    .expect("pass");

                let plum_sig_3_plum_head_seal =
                    idp_sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
                        plum_sig_2_plum_head_seal,
                        // NOTE that the signer is now signer_1, which must match the previous OwnedData's owner.
                        &signer_1_priv_jwk,
                        signer_1_did.clone(),
                        content_plum_head_seal_v[3].clone(),
                        &mut datahost,
                        None,
                    )
                    .block_on()
                    .expect("pass");
                // Update the PathState.
                idp_sig::execute_path_state_plum_sig_update(
                    &mut datahost,
                    None,
                    path.clone(),
                    plum_sig_3_plum_head_seal.clone(),
                )
                .block_on()
                .expect("pass");
                // Verify it.
                assert_eq!(
                    datahost
                        .load_path_state(&path, None)
                        .block_on()
                        .expect("pass"),
                    idp_proto::PathState {
                        path: path.clone(),
                        current_state_plum_head_seal: plum_sig_3_plum_head_seal.clone()
                    }
                );
                idp_sig::PlumSig::verify_chain(&plum_sig_3_plum_head_seal, &mut datahost, None)
                    .block_on()
                    .expect("pass");
            }
            // Make a PlumSig with an invalid signature
            if true {
                let legit_signer_priv_jwk = idp_sig::KeyType::Secp256k1
                    .generate_priv_jwk()
                    .expect("pass");
                let legit_signer_pub_jwk = legit_signer_priv_jwk.to_public();
                let _legit_signer_did = idp_sig::did_key_from_jwk(&legit_signer_pub_jwk)
                    .expect("pass")
                    .did;

                let plum_head_seal = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_metadata_nonce(Nonce::generate())
                            .with_plum_created_at(UnixNanoseconds::now())
                            .with_plum_relations_and_plum_body_content_from(
                                &"here's some fraudulent data!".to_string(),
                                None,
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();

                // Create a legit PlumSig.
                let legit_plum_sig_p =
                    idp_sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_without_previous(
                        &legit_signer_priv_jwk,
                        plum_head_seal.clone(),
                        &mut datahost,
                        None,
                    )
                    .block_on()
                    .unwrap();
                // Load the PlumSig and its OwnedData so that it can be picked apart to attempt to
                // construct a fraudulent PlumSig.
                let legit_plum_sig: PlumSig = datahost
                    .load_plum_and_decode_and_deserialize(&legit_plum_sig_p, None)
                    .block_on()
                    .unwrap();
                let legit_owned_data: OwnedData = datahost
                    .load_plum_and_decode_and_deserialize(&legit_plum_sig.content.plum, None)
                    .block_on()
                    .unwrap();

                // Create a key for the attacker
                let attacker_signer_priv_jwk = idp_sig::KeyType::Secp256k1
                    .generate_priv_jwk()
                    .expect("pass");
                let attacker_signer_pub_jwk = attacker_signer_priv_jwk.to_public();
                let attacker_signer_did = idp_sig::did_key_from_jwk(&attacker_signer_pub_jwk)
                    .expect("pass")
                    .did;

                let fradulent_owned_data = OwnedData {
                    owner: attacker_signer_did.clone(),
                    data: legit_owned_data.data,
                    previous_owned_data_o: legit_owned_data.previous_owned_data_o,
                };
                let fradulent_owned_data_p = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_relations_and_plum_body_content_from(
                                &fradulent_owned_data,
                                Some(&idp_proto::ContentFormat::json()),
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();
                // By construction it's difficult to attempt to create a fraudulent PlumSig,
                // so this attempt is contrived and silly.
                let fradulent_plum_sig = PlumSig {
                    content: PlumSigContent {
                        nonce: legit_plum_sig.content.nonce,
                        plum: fradulent_owned_data_p,
                        previous_plum_sig_o: legit_plum_sig.content.previous_plum_sig_o,
                    },
                    signature: legit_plum_sig.signature,
                };
                let fraudulent_plum_sig_p = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_relations_and_plum_body_content_from(
                                &fradulent_plum_sig,
                                Some(&idp_proto::ContentFormat::json()),
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();
                fradulent_plum_sig
                    .verify_against_known_signer(&legit_signer_pub_jwk)
                    .expect_err("pass");
                fradulent_plum_sig
                    .verify_against_known_signer(&attacker_signer_pub_jwk)
                    .expect_err("pass");
                fradulent_plum_sig
                    .verify_and_extract_signer()
                    .block_on()
                    .expect_err("pass");
                PlumSig::verify_chain(&fraudulent_plum_sig_p, &mut datahost, None)
                    .block_on()
                    .expect_err("pass");
            }
            // Make some DirNode content
            if true {
                let plum0_head_seal = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_relations_and_plum_body_content_from(
                                &"how do ostriches and hippos stack up?".to_string(),
                                None,
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();
                let plum1_head_seal = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_relations_and_plum_body_content_from(
                                &"they are arch-rivals.".to_string(),
                                None,
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();
                let _dir_node_plum_head_seal = datahost
                    .store_plum(
                        &idp_proto::PlumBuilder::new()
                            .with_plum_relations_and_plum_body_content_from(
                                &DirNode {
                                    entry_m: maplit::btreemap! {
                                        "question".to_string() => plum0_head_seal,
                                        "answer".to_string() => plum1_head_seal,
                                    },
                                },
                                Some(&idp_proto::ContentFormat::json()),
                                idp_proto::ContentEncoding::none(),
                            )
                            .unwrap()
                            .build()
                            .unwrap(),
                        None,
                    )
                    .block_on()
                    .unwrap();
            }
        }

        let view_stack_v = Vec::new();
        Self {
            path_state_table_view: PathStateTableView::new(),
            plum_table_view: PlumTableView::new(),
            view_stack_v,
            debug: false,
            datahost,
        }
    }
}
