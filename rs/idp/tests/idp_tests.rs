use anyhow::Result;
use idp_client::IDPClient;
use idp_core::{BranchNode, Datahost, FragmentQueryResult, FragmentQueryable};
use idp_proto::{ContentType, Plum, PlumBuilder, PlumHeadSeal, RelationFlags};
use idp_server::IDPServer;
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestData {
    pub datahost_la: Arc<RwLock<Datahost>>,
    pub content_1: String,
    pub content_2: String,
    pub content_1_plum: Plum,
    pub content_2_plum: Plum,
    pub metadata_0_plum: Plum,
    pub metadata_1_plum: Plum,
    pub metadata_2_plum: Plum,
    pub content_1_plum_head_seal: PlumHeadSeal,
    pub content_2_plum_head_seal: PlumHeadSeal,
    pub metadata_0_plum_head_seal: PlumHeadSeal,
    pub metadata_1_plum_head_seal: PlumHeadSeal,
    pub metadata_2_plum_head_seal: PlumHeadSeal,
    pub branch_node_0: BranchNode,
    pub branch_node_0_plum: Plum,
    pub branch_node_0_plum_head_seal: PlumHeadSeal,
    pub branch_node_1: BranchNode,
    pub branch_node_1_plum: Plum,
    pub branch_node_1_plum_head_seal: PlumHeadSeal,
    pub branch_node_2: BranchNode,
    pub branch_node_2_plum: Plum,
    pub branch_node_2_plum_head_seal: PlumHeadSeal,
}

impl TestData {
    pub fn create(datahost_la: Arc<RwLock<Datahost>>) -> Self {
        let datahost_g = datahost_la.read();

        let content_1 = format!("splunges are cool, {}", Uuid::new_v4());
        let content_2 = format!("HIPPOS are cool, {}", Uuid::new_v4());

        let content_1_plum = PlumBuilder::new()
            .with_body_content_type(ContentType::from("text/plain"))
            .with_body_content(content_1.as_bytes().to_vec())
            .build()
            .expect("pass");
        let content_2_plum = PlumBuilder::new()
            .with_body_content_type(ContentType::from("text/plain"))
            .with_body_content(content_2.as_bytes().to_vec())
            .build()
            .expect("pass");

        let metadata_0_plum = PlumBuilder::new()
            .with_body_content_type(ContentType::from("text/plain"))
            .with_body_content(
                format!("Branch root, {}", Uuid::new_v4())
                    .as_bytes()
                    .to_vec(),
            )
            .build()
            .expect("pass");
        let metadata_1_plum = PlumBuilder::new()
            .with_body_content_type(ContentType::from("text/plain"))
            .with_body_content(
                format!("Initial statement, {}", Uuid::new_v4())
                    .as_bytes()
                    .to_vec(),
            )
            .build()
            .expect("pass");
        let metadata_2_plum = PlumBuilder::new()
            .with_body_content_type(ContentType::from("text/plain"))
            .with_body_content(
                format!(
                    "Revised statement authored by the HIPPO lobby, {}",
                    Uuid::new_v4()
                )
                .as_bytes()
                .to_vec(),
            )
            .build()
            .expect("pass");

        let content_1_plum_head_seal = datahost_g.store_plum(&content_1_plum).expect("pass");
        let content_2_plum_head_seal = datahost_g.store_plum(&content_2_plum).expect("pass");

        log::trace!("content_1_plum_head_seal: {}", content_1_plum_head_seal);
        log::trace!("content_2_plum_head_seal: {}", content_2_plum_head_seal);

        let metadata_0_plum_head_seal = datahost_g.store_plum(&metadata_0_plum).expect("pass");
        let metadata_1_plum_head_seal = datahost_g.store_plum(&metadata_1_plum).expect("pass");
        let metadata_2_plum_head_seal = datahost_g.store_plum(&metadata_2_plum).expect("pass");

        log::trace!("metadata_0_plum_head_seal: {}", metadata_0_plum_head_seal);
        log::trace!("metadata_1_plum_head_seal: {}", metadata_1_plum_head_seal);
        log::trace!("metadata_2_plum_head_seal: {}", metadata_2_plum_head_seal);

        let branch_node_0 = BranchNode {
            ancestor_o: None,
            metadata: metadata_0_plum_head_seal.clone(),
            content_o: None,
            posi_diff_o: None,
            nega_diff_o: None,
        };
        let branch_node_0_plum = PlumBuilder::new()
            .with_relational_typed_content_from(&branch_node_0)
            .expect("pass")
            .build()
            .expect("pass");
        let branch_node_0_plum_head_seal =
            datahost_g.store_plum(&branch_node_0_plum).expect("pass");

        let branch_node_1 = BranchNode {
            ancestor_o: Some(branch_node_0_plum_head_seal.clone()),
            metadata: metadata_1_plum_head_seal.clone(),
            content_o: Some(content_1_plum_head_seal.clone()),
            posi_diff_o: None,
            nega_diff_o: None,
        };
        let branch_node_1_plum = PlumBuilder::new()
            .with_relational_typed_content_from(&branch_node_1)
            .expect("pass")
            .build()
            .expect("pass");
        let branch_node_1_plum_head_seal =
            datahost_g.store_plum(&branch_node_1_plum).expect("pass");

        let branch_node_2 = BranchNode {
            ancestor_o: Some(branch_node_1_plum_head_seal.clone()),
            metadata: metadata_2_plum_head_seal.clone(),
            content_o: Some(content_2_plum_head_seal.clone()),
            posi_diff_o: None,
            nega_diff_o: None,
        };
        let branch_node_2_plum = PlumBuilder::new()
            .with_relational_typed_content_from(&branch_node_2)
            .expect("pass")
            .build()
            .expect("pass");
        let branch_node_2_plum_head_seal =
            datahost_g.store_plum(&branch_node_2_plum).expect("pass");

        log::trace!(
            "branch_node_0_plum_head_seal: {}",
            branch_node_0_plum_head_seal
        );
        log::trace!(
            "branch_node_1_plum_head_seal: {}",
            branch_node_1_plum_head_seal
        );
        log::trace!(
            "branch_node_2_plum_head_seal: {}",
            branch_node_2_plum_head_seal
        );

        //
        // Now accumulate_relations_recursive and check the results.  branch_node_2_plum is the head
        // of the branch, so it should depend on all other Plums.
        // TODO: Actually check the RelationFlags values
        //

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&content_1_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            assert!(relation_flags_m.is_empty());
        }
        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&content_2_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            assert!(relation_flags_m.is_empty());
        }

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&metadata_0_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            assert!(relation_flags_m.is_empty());
        }
        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&metadata_1_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            assert!(relation_flags_m.is_empty());
        }
        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&metadata_2_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            assert!(relation_flags_m.is_empty());
        }

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&branch_node_0_plum_head_seal, RelationFlags::ALL)
                .expect("pass");
            log::debug!("relation_flags_m: {:?}", relation_flags_m);
            assert_eq!(relation_flags_m.len(), 1);
            assert!(relation_flags_m.contains_key(&metadata_0_plum_head_seal));
        }

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &branch_node_0_plum_head_seal,
                    RelationFlags::CONTENT_DEPENDENCY,
                )
                .expect("pass");
            log::debug!("relation_flags_m: {:?}", relation_flags_m);
            // Empty because metadata is METADATA_DEPENDENCY.
            assert!(relation_flags_m.is_empty());
        }

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&branch_node_1_plum_head_seal, RelationFlags::ALL)
                .expect("pass");

            log::debug!("relation_flags_m: {:?}", relation_flags_m);
            assert_eq!(relation_flags_m.len(), 4);
            // These are the dependencies of branch_node_0_plum
            assert!(relation_flags_m.contains_key(&metadata_0_plum_head_seal));
            // These are the dependencies of branch_node_1_plum
            assert!(relation_flags_m.contains_key(&branch_node_0_plum_head_seal));
            assert!(relation_flags_m.contains_key(&metadata_1_plum_head_seal));
            assert!(relation_flags_m.contains_key(&content_1_plum_head_seal));
        }

        {
            let relation_flags_m = datahost_g
                .accumulated_relations_recursive(&branch_node_2_plum_head_seal, RelationFlags::ALL)
                .expect("pass");

            log::debug!("relation_flags_m: {:?}", relation_flags_m);
            assert_eq!(relation_flags_m.len(), 7);
            // These are the dependencies of branch_node_0_plum
            assert!(relation_flags_m.contains_key(&metadata_0_plum_head_seal));
            // These are the dependencies of branch_node_1_plum
            assert!(relation_flags_m.contains_key(&branch_node_0_plum_head_seal));
            assert!(relation_flags_m.contains_key(&metadata_1_plum_head_seal));
            assert!(relation_flags_m.contains_key(&content_1_plum_head_seal));
            // These are the dependencies of branch_node_2_plum
            assert!(relation_flags_m.contains_key(&branch_node_1_plum_head_seal));
            assert!(relation_flags_m.contains_key(&metadata_2_plum_head_seal));
            assert!(relation_flags_m.contains_key(&content_2_plum_head_seal));
        }

        //
        // Testing FragmentQueryable
        //

        {
            // Self-query
            assert_eq!(
                branch_node_0
                    .fragment_query_single_segment(&branch_node_0_plum_head_seal, "")
                    .expect("pass"),
                FragmentQueryResult::Value(branch_node_0_plum_head_seal.clone()),
            );
            // No ancestor
            assert!(branch_node_0
                .fragment_query_single_segment(&branch_node_0_plum_head_seal, "ancestor")
                .is_err());
            // No content
            assert!(branch_node_0
                .fragment_query_single_segment(&branch_node_0_plum_head_seal, "content")
                .is_err());
            // Invalid entry
            assert!(branch_node_0
                .fragment_query_single_segment(&branch_node_0_plum_head_seal, "nonexistent")
                .is_err());
            assert_eq!(
                branch_node_0
                    .fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata")
                    .expect("pass"),
                FragmentQueryResult::Value(metadata_0_plum_head_seal.clone()),
            );
            assert_eq!(
                branch_node_0
                    .fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata/")
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_0_plum_head_seal.clone(),
                    rest_of_query_str: ""
                },
            );
            assert_eq!(
                branch_node_0
                    .fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata/stuff")
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_0_plum_head_seal.clone(),
                    rest_of_query_str: "stuff"
                },
            );
            assert_eq!(
                branch_node_0
                    .fragment_query_single_segment(
                        &branch_node_0_plum_head_seal,
                        "metadata/stuff/and/things"
                    )
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_0_plum_head_seal.clone(),
                    rest_of_query_str: "stuff/and/things"
                },
            );
        }

        {
            // Self-query
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "")
                    .expect("pass"),
                FragmentQueryResult::Value(branch_node_1_plum_head_seal.clone()),
            );
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "ancestor")
                    .expect("pass"),
                FragmentQueryResult::Value(branch_node_0_plum_head_seal.clone()),
            );
            // No content
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "content")
                    .expect("pass"),
                FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
            );
            // Invalid entry
            assert!(branch_node_1
                .fragment_query_single_segment(&branch_node_1_plum_head_seal, "nonexistent")
                .is_err());
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata")
                    .expect("pass"),
                FragmentQueryResult::Value(metadata_1_plum_head_seal.clone()),
            );
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata/")
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_1_plum_head_seal.clone(),
                    rest_of_query_str: ""
                },
            );
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata/stuff")
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_1_plum_head_seal.clone(),
                    rest_of_query_str: "stuff"
                },
            );
            assert_eq!(
                branch_node_1
                    .fragment_query_single_segment(
                        &branch_node_1_plum_head_seal,
                        "metadata/stuff/and/things"
                    )
                    .expect("pass"),
                FragmentQueryResult::ForwardQueryTo {
                    target: metadata_1_plum_head_seal.clone(),
                    rest_of_query_str: "stuff/and/things"
                },
            );
        }

        Self {
            datahost_la: datahost_la.clone(),
            content_1,
            content_2,
            content_1_plum,
            content_2_plum,
            metadata_0_plum,
            metadata_1_plum,
            metadata_2_plum,
            content_1_plum_head_seal,
            content_2_plum_head_seal,
            metadata_0_plum_head_seal,
            metadata_1_plum_head_seal,
            metadata_2_plum_head_seal,
            branch_node_0,
            branch_node_0_plum,
            branch_node_0_plum_head_seal,
            branch_node_1,
            branch_node_1_plum,
            branch_node_1_plum_head_seal,
            branch_node_2,
            branch_node_2_plum,
            branch_node_2_plum_head_seal,
        }
    }
}

async fn run_client_task(client_datahost_la: Arc<RwLock<Datahost>>) -> Result<TestData> {
    // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
    // to finish spinning up.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
    //     .connect()
    //     .await?;

    // let mut idp_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
    //     let token = tonic::metadata::MetadataValue::from_str("Bearer some-secret-token").unwrap();
    //     move |mut req: tonic::Request<()>| {
    //         req.metadata_mut().insert("authorization", token.clone());
    //         Ok(req)
    //     }
    // });

    let test_data = TestData::create(client_datahost_la.clone());
    let mut idp_client = IDPClient::connect(client_datahost_la.clone()).await?;

    // Happy path
    {
        idp_client
            .push(&test_data.branch_node_2_plum_head_seal)
            .await?;
    }

    Ok(test_data)
}

async fn run_server_task(server_datahost_la: Arc<RwLock<Datahost>>) -> Result<()> {
    let addr = "[::1]:50051".parse().unwrap();
    let idp_server = IDPServer::new(server_datahost_la);
    idp_server.listen_on(addr).await?;
    Ok(())
}

#[tokio::test]
async fn test_client_server() {
    let _ = env_logger::try_init();

    let client_datahost_la = Arc::new(RwLock::new(
        Datahost::open_in_memory("client datahost".to_string()).expect("pass"),
    ));
    let server_datahost_la = Arc::new(RwLock::new(
        Datahost::open_in_memory("server datahost".to_string()).expect("pass"),
    ));

    let server_handle = tokio::spawn(run_server_task(server_datahost_la.clone()));
    let client_handle = tokio::spawn(run_client_task(client_datahost_la.clone()));

    // Wait on the client task to finish.
    let client_created_test_data_r = tokio::join!(client_handle).0.expect("pass");
    // The client is done, so stop the server.
    server_handle.abort();
    // Handle the client result.
    let client_created_test_data = client_created_test_data_r.expect("pass");

    // Now make sure that server_datahost_la has the expected Plum-s.
    {
        let server_datahost_g = server_datahost_la.read();

        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.branch_node_0_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.branch_node_1_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.branch_node_2_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.content_1_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.content_2_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.metadata_0_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.metadata_1_plum_head_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_head_row(&client_created_test_data.metadata_2_plum_head_seal)
            .expect("pass")
            .is_some());

        assert!(server_datahost_g
            .select_option_plum_body_row(
                &client_created_test_data.branch_node_0_plum.head.body_seal
            )
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(
                &client_created_test_data.branch_node_1_plum.head.body_seal
            )
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(
                &client_created_test_data.branch_node_2_plum.head.body_seal
            )
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(&client_created_test_data.content_1_plum.head.body_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(&client_created_test_data.content_2_plum.head.body_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(&client_created_test_data.metadata_0_plum.head.body_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(&client_created_test_data.metadata_1_plum.head.body_seal)
            .expect("pass")
            .is_some());
        assert!(server_datahost_g
            .select_option_plum_body_row(&client_created_test_data.metadata_2_plum.head.body_seal)
            .expect("pass")
            .is_some());
    }
}
