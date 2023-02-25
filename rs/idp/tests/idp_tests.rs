use async_lock::RwLock;
use idp::{
    core::{
        BranchNode, Datacache, Datahost, FragmentQueryResult, FragmentQueryable, IDPClient,
        PlumRef, PlumURI, PlumURIRemote,
    },
    datahost_storage_sqlite::DatahostStorageSQLite,
    proto::{ContentType, Plum, PlumBuilder, PlumHeadSeal, PlumRelationFlags},
    server::IDPServer,
};
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
    pub async fn create(datahost_la: Arc<RwLock<Datahost>>) -> Self {
        let datahost_g = datahost_la.read().await;

        let content_1 = format!("splunges are cool, {}", Uuid::new_v4());
        let content_2 = format!("HIPPOS are cool, {}", Uuid::new_v4());

        let content_1_plum = PlumBuilder::new()
            .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
            .with_plum_body_content(content_1.as_bytes().to_vec())
            .build()
            .expect("pass");
        let content_2_plum = PlumBuilder::new()
            .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
            .with_plum_body_content(content_2.as_bytes().to_vec())
            .build()
            .expect("pass");

        let metadata_0_plum = PlumBuilder::new()
            .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
            .with_plum_body_content(
                format!("Branch root, {}", Uuid::new_v4())
                    .as_bytes()
                    .to_vec(),
            )
            .build()
            .expect("pass");
        let metadata_1_plum = PlumBuilder::new()
            .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
            .with_plum_body_content(
                format!("Initial statement, {}", Uuid::new_v4())
                    .as_bytes()
                    .to_vec(),
            )
            .build()
            .expect("pass");
        let metadata_2_plum = PlumBuilder::new()
            .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
            .with_plum_body_content(
                format!(
                    "Revised statement authored by the HIPPO lobby, {}",
                    Uuid::new_v4()
                )
                .as_bytes()
                .to_vec(),
            )
            .build()
            .expect("pass");

        let content_1_plum_head_seal = datahost_g
            .store_plum(&content_1_plum, None)
            .await
            .expect("pass");
        let content_2_plum_head_seal = datahost_g
            .store_plum(&content_2_plum, None)
            .await
            .expect("pass");

        log::trace!("content_1_plum_head_seal: {}", content_1_plum_head_seal);
        log::trace!("content_2_plum_head_seal: {}", content_2_plum_head_seal);

        let metadata_0_plum_head_seal = datahost_g
            .store_plum(&metadata_0_plum, None)
            .await
            .expect("pass");
        let metadata_1_plum_head_seal = datahost_g
            .store_plum(&metadata_1_plum, None)
            .await
            .expect("pass");
        let metadata_2_plum_head_seal = datahost_g
            .store_plum(&metadata_2_plum, None)
            .await
            .expect("pass");

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
        let branch_node_0_plum_head_seal = datahost_g
            .store_plum(&branch_node_0_plum, None)
            .await
            .expect("pass");

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
        let branch_node_1_plum_head_seal = datahost_g
            .store_plum(&branch_node_1_plum, None)
            .await
            .expect("pass");

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
        let branch_node_2_plum_head_seal = datahost_g
            .store_plum(&branch_node_2_plum, None)
            .await
            .expect("pass");

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
        // TODO: Actually check the PlumRelationFlags values
        //

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &content_1_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            assert!(plum_relation_flags_m.is_empty());
        }
        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &content_2_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            assert!(plum_relation_flags_m.is_empty());
        }

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &metadata_0_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            assert!(plum_relation_flags_m.is_empty());
        }
        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &metadata_1_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            assert!(plum_relation_flags_m.is_empty());
        }
        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &metadata_2_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            assert!(plum_relation_flags_m.is_empty());
        }

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &branch_node_0_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");
            log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
            assert_eq!(plum_relation_flags_m.len(), 1);
            assert!(plum_relation_flags_m.contains_key(&metadata_0_plum_head_seal));
        }

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &branch_node_0_plum_head_seal,
                    PlumRelationFlags::CONTENT_DEPENDENCY,
                    None,
                )
                .await
                .expect("pass");
            log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
            // Empty because metadata is METADATA_DEPENDENCY.
            assert!(plum_relation_flags_m.is_empty());
        }

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &branch_node_1_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");

            log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
            assert_eq!(plum_relation_flags_m.len(), 4);
            // These are the dependencies of branch_node_0_plum
            assert!(plum_relation_flags_m.contains_key(&metadata_0_plum_head_seal));
            // These are the dependencies of branch_node_1_plum
            assert!(plum_relation_flags_m.contains_key(&branch_node_0_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&metadata_1_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&content_1_plum_head_seal));
        }

        {
            let plum_relation_flags_m = datahost_g
                .accumulated_relations_recursive(
                    &branch_node_2_plum_head_seal,
                    PlumRelationFlags::ALL,
                    None,
                )
                .await
                .expect("pass");

            log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
            assert_eq!(plum_relation_flags_m.len(), 7);
            // These are the dependencies of branch_node_0_plum
            assert!(plum_relation_flags_m.contains_key(&metadata_0_plum_head_seal));
            // These are the dependencies of branch_node_1_plum
            assert!(plum_relation_flags_m.contains_key(&branch_node_0_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&metadata_1_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&content_1_plum_head_seal));
            // These are the dependencies of branch_node_2_plum
            assert!(plum_relation_flags_m.contains_key(&branch_node_1_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&metadata_2_plum_head_seal));
            assert!(plum_relation_flags_m.contains_key(&content_2_plum_head_seal));
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
    async fn check_target_datahost(&self, target_name: &str, target_datahost: &Datahost) {
        assert!(
            target_datahost
                .has_plum_head(&self.branch_node_2_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.branch_node_2_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.branch_node_2_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.branch_node_2_plum.plum_head.plum_body_seal
        );
        assert!(
            target_datahost
                .has_plum_head(&self.content_2_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.content_2_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.content_2_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.content_2_plum.plum_head.plum_body_seal
        );
        assert!(
            target_datahost
                .has_plum_head(&self.metadata_2_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.metadata_2_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.metadata_2_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.metadata_2_plum.plum_head.plum_body_seal
        );

        //
        // 1
        //

        assert!(
            target_datahost
                .has_plum_head(&self.branch_node_1_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.branch_node_1_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.branch_node_1_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.branch_node_1_plum.plum_head.plum_body_seal
        );
        assert!(
            target_datahost
                .has_plum_head(&self.content_1_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.content_1_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.content_1_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.content_1_plum.plum_head.plum_body_seal
        );
        assert!(
            target_datahost
                .has_plum_head(&self.metadata_1_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.metadata_1_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.metadata_1_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.metadata_1_plum.plum_head.plum_body_seal
        );

        //
        // 0
        //

        assert!(
            target_datahost
                .has_plum_head(&self.branch_node_0_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.branch_node_0_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.branch_node_0_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.branch_node_0_plum.plum_head.plum_body_seal
        );
        assert!(
            target_datahost
                .has_plum_head(&self.metadata_0_plum_head_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumHead {}",
            target_name,
            self.metadata_0_plum_head_seal
        );
        assert!(
            target_datahost
                .has_plum_body(&self.metadata_0_plum.plum_head.plum_body_seal, None)
                .await
                .expect("pass"),
            "{} is missing PlumBody {}",
            target_name,
            self.metadata_0_plum.plum_head.plum_body_seal
        );
    }
}

// async fn run_client_task_for_push(client_datahost_la: Arc<RwLock<Datahost>>) -> TestData {
//     // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
//     // to finish spinning up.
//     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

//     // let channel = tonic::transport::Channel::from_static("http://0.0.0.0:50051")
//     //     .connect()
//     //     .await?;

//     // let mut idp_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
//     //     let token = tonic::metadata::MetadataValue::from_str("Bearer some-secret-token").unwrap();
//     //     move |mut req: tonic::Request<()>| {
//     //         req.metadata_mut().insert("authorization", token.clone());
//     //         Ok(req)
//     //     }
//     // });

//     let test_data = TestData::create(client_datahost_la.clone()).await;
//     let mut idp_client =
//         IDPClient::connect("http://0.0.0.0:50051".to_string(), client_datahost_la.clone())
//             .await
//             .expect("pass");

//     // Happy path
//     {
//         log::info!(
//             "client is pushing {}",
//             test_data.branch_node_2_plum_head_seal
//         );
//         idp_client
//             .push(&test_data.branch_node_2_plum_head_seal)
//             .await
//             .expect("pass");
//     }

//     test_data
// }

// async fn run_client_task_for_pull(client_datahost_la: Arc<RwLock<Datahost>>, plum_head_seal: &PlumHeadSeal) {
//     // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
//     // to finish spinning up.
//     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

//     // let channel = tonic::transport::Channel::from_static("http://0.0.0.0:50051")
//     //     .connect()
//     //     .await?;

//     // let mut idp_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
//     //     let token = tonic::metadata::MetadataValue::from_str("Bearer some-secret-token").unwrap();
//     //     move |mut req: tonic::Request<()>| {
//     //         req.metadata_mut().insert("authorization", token.clone());
//     //         Ok(req)
//     //     }
//     // });

//     let mut idp_client =
//         IDPClient::connect("http://0.0.0.0:50051".to_string(), client_datahost_la.clone())
//             .await
//             .expect("pass");

//     // Happy path
//     {
//         log::info!(
//             "client is pulling {}",
//             plum_head_seal
//         );
//         idp_client
//             .pull(plum_head_seal)
//             .await
//             .expect("pass");
//     }
// }

// async fn run_server_task_for_push(server_datahost_la: Arc<RwLock<Datahost>>) -> Result<()> {
//     let addr = "0.0.0.0:50051".parse().unwrap();
//     let idp_server = IDPServer::new(server_datahost_la);
//     idp_server.listen_on(addr).await?;
//     Ok(())
// }

// async fn run_server_task_for_pull(server_datahost_la: Arc<RwLock<Datahost>>) -> Result<()> {
//     let addr = "0.0.0.0:50051".parse().unwrap();
//     let idp_server = IDPServer::new(server_datahost_la);
//     idp_server.listen_on(addr).await?;
//     Ok(())
// }

#[tokio::test]
#[serial_test::serial]
async fn test_client_server_push() {
    let _ = env_logger::try_init();

    // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
    let client_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_client.db?mode=rwc")
            .await
            .expect("pass"),
    )));
    let client_created_test_data = TestData::create(client_datahost_la.clone()).await;

    let server_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_server.db?mode=rwc")
            .await
            .expect("pass"),
    )));

    let client_handle = {
        let client_datahost_la = client_datahost_la.clone();
        let branch_node_2_plum_head_seal = client_created_test_data
            .branch_node_2_plum_head_seal
            .clone();
        tokio::spawn(async move {
            // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
            // to finish spinning up.
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let mut idp_client = IDPClient::connect(
                "http://0.0.0.0:50051".to_string(),
                client_datahost_la.clone(),
            )
            .await
            .expect("pass");

            // Push the head of the branch.
            // log::info!(
            //     "client is pushing {}",
            //     test_data.branch_node_2_plum_head_seal
            // );
            idp_client
                .push(&branch_node_2_plum_head_seal)
                .await
                .expect("pass");
        })
    };
    let server_handle = {
        let server_datahost_la = server_datahost_la.clone();
        tokio::spawn(async move {
            let addr = "0.0.0.0:50051".parse().unwrap();
            let idp_server = IDPServer::new(server_datahost_la);
            idp_server.listen_on(addr).await.expect("pass");
        })
    };

    // Wait on the client task to finish.
    tokio::join!(client_handle).0.expect("pass");
    // Wait a while for the server to catch up (maybe could try to get a write lock on server_datahost_la to do this)
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // The client is done, so stop the server.
    server_handle.abort();

    // Now make sure that server_datahost_la has the expected Plum-s.
    client_created_test_data
        .check_target_datahost("server", &*server_datahost_la.read().await)
        .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_client_server_pull() {
    let _ = env_logger::try_init();

    // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
    let client_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_client.db?mode=rwc")
            .await
            .expect("pass"),
    )));

    let server_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_server.db?mode=rwc")
            .await
            .expect("pass"),
    )));
    let server_created_test_data = TestData::create(server_datahost_la.clone()).await;

    let client_handle = {
        let client_datahost_la = client_datahost_la.clone();
        let branch_node_2_plum_head_seal = server_created_test_data
            .branch_node_2_plum_head_seal
            .clone();
        tokio::spawn(async move {
            // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
            // to finish spinning up.
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let mut idp_client = IDPClient::connect(
                "http://0.0.0.0:50051".to_string(),
                client_datahost_la.clone(),
            )
            .await
            .expect("pass");

            // Push the head of the branch.
            idp_client
                .pull(&branch_node_2_plum_head_seal)
                .await
                .expect("pass");
        })
    };
    let server_handle = {
        let server_datahost_la = server_datahost_la.clone();
        tokio::spawn(async move {
            let addr = "0.0.0.0:50051".parse().unwrap();
            let idp_server = IDPServer::new(server_datahost_la);
            idp_server.listen_on(addr).await.expect("pass");
        })
    };

    // Wait on the client task to finish.
    tokio::join!(client_handle).0.expect("pass");
    // Wait a while for the server to catch up (maybe could try to get a write lock on server_datahost_la to do this)
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // The client is done, so stop the server.
    server_handle.abort();

    // Now make sure that client_datahost_la has the expected Plum-s.
    server_created_test_data
        .check_target_datahost("client", &*client_datahost_la.read().await)
        .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_client_server_plum_ref() {
    let _ = env_logger::try_init();

    // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
    let client_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_client.db?mode=rwc")
            .await
            .expect("pass"),
    )));
    // Have to set the Datacache singleton with a Datacache attached to the client Datahost.
    Datacache::set_singleton(Box::new(Datacache::new(client_datahost_la.clone())));

    let server_datahost_la = Arc::new(RwLock::new(Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations("sqlite:idp_tests_server.db?mode=rwc")
            .await
            .expect("pass"),
    )));
    let server_created_test_data = TestData::create(server_datahost_la.clone()).await;

    let client_handle = {
        let branch_node_2_plum_head_seal = server_created_test_data
            .branch_node_2_plum_head_seal
            .clone();
        tokio::spawn(async move {
            // Wait for the server to finish spinning up.  TODO: Instead, use a channel that waits for the server
            // to finish spinning up.
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Create a PlumRef pointing at branch_node_2_plum_head_seal on the server.
            let plum_ref = PlumRef::<BranchNode>::new(PlumURI::Remote(PlumURIRemote {
                hostname: "0.0.0.0".to_string(),
                port_o: Some(50051),
                plum_head_seal: branch_node_2_plum_head_seal,
            }));
            // This should cause a pull of branch_node_2_plum_head_seal.
            let _value_a = plum_ref.value_a().await.expect("pass");
            log::info!("client's {} was retrieved", plum_ref);
        })
    };
    let server_handle = {
        let server_datahost_la = server_datahost_la.clone();
        tokio::spawn(async move {
            let addr = "0.0.0.0:50051".parse().unwrap();
            let idp_server = IDPServer::new(server_datahost_la);
            idp_server.listen_on(addr).await.expect("pass");
        })
    };

    // Wait on the client task to finish.
    tokio::join!(client_handle).0.expect("pass");
    // Wait a while for the server to catch up (maybe could try to get a write lock on server_datahost_la to do this)
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // The client is done, so stop the server.
    server_handle.abort();

    // Now make sure that client_datahost_la has the expected Plum-s.
    server_created_test_data
        .check_target_datahost("client", &*client_datahost_la.read().await)
        .await;
}
