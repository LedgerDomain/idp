use async_lock::RwLock;
use idp_core::{
    BranchNode, BranchNodeBuilder, Datacache, Datahost, DirNode, FragmentQueryResult,
    FragmentQueryable, PlumRef, PlumURI, PlumURILocal,
};
use idp_datahost_storage_sqlite::DatahostStorageSQLite;
use idp_proto::{
    branch_set_head_request, BranchSetHeadRequest, ContentEncoding, ContentFormat, Path, PathState,
    Plum, PlumBodySeal, PlumBuilder, PlumHeadSeal, PlumRelationFlags, Sha256Sum,
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

/// This will run once at load time (i.e. presumably before main function is called).
#[ctor::ctor]
fn overall_init() {
    env_logger::init();
}

/// Convenience function for opening the DB whose URL is specified by the DATABASE_URL env var.
/// If that var isn't set, then it defaults to using an in-memory DB.
async fn datahost_from_env_var() -> Datahost {
    // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
    let database_url = "sqlite:idp_core_tests.db?mode=rwc";
    log::info!(
        "datahost_from_env_var is using {:?} as database_url",
        database_url
    );
    Datahost::open(
        DatahostStorageSQLite::connect_and_run_migrations(database_url)
            .await
            .expect("pass"),
    )
}

#[test]
fn display_sha256sum() -> anyhow::Result<()> {
    // Yes, this isn't actually a 256 bit value, but who cares.
    let sha256sum = Sha256Sum::from(vec![0xAB, 0x91, 0xCE]);
    log::debug!("sha256sum: {}", sha256sum);
    let sha256sum_string = sha256sum.to_string();
    assert_eq!(sha256sum_string.as_str(), "AB91CE");

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn open_datahost() {
    datahost_from_env_var().await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_create_plum_head() {
    let plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("test_datahost_create_plum_head, {}.", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let head_seal = datahost
        .store_plum_head(&plum.plum_head, None)
        .await
        .expect("pass");
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.plum_head));

    // store_plum_head again and ensure it worked again.
    let head_seal_2 = datahost
        .store_plum_head(&plum.plum_head, None)
        .await
        .expect("pass");
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum.plum_head));

    // load_plum_head and check that it matches.
    let plum_head_2 = datahost
        .load_plum_head(&head_seal, None)
        .await
        .expect("pass");
    assert_eq!(plum_head_2, plum.plum_head);

    // log::debug!(
    //     "reference count for {:?} is {}",
    //     plum.plum_head.plum_body_seal,
    //     datahost
    //         .select_plum_body_reference_count(&plum.plum_head.plum_body_seal)
    //         .await
    //         .expect("pass")
    // );
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_create_plum_body() {
    let plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("test_datahost_create_plum_body, {}.", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let plum_body_seal = datahost
        .store_plum_body(&plum.plum_body, None)
        .await
        .expect("pass");
    assert_eq!(plum_body_seal, PlumBodySeal::from(&plum.plum_body));

    // store_plum_body again and ensure it worked again.
    let body_seal_2 = datahost
        .store_plum_body(&plum.plum_body, None)
        .await
        .expect("pass");
    assert_eq!(body_seal_2, PlumBodySeal::from(&plum.plum_body));

    // log::debug!(
    //     "reference count for {:?} is {}",
    //     plum_body_seal,
    //     datahost
    //         .select_plum_body_reference_count(&plum_body_seal)
    //         .await
    //         .expect("pass")
    // );
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_create_plum() {
    let plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("test_datahost_create_plum, {}.", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let head_seal = datahost.store_plum(&plum, None).await.expect("pass");
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.plum_head));

    // store_plum again and ensure it worked again
    let head_seal_2 = datahost.store_plum(&plum, None).await.expect("pass");
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum.plum_head));

    // log::debug!(
    //     "reference count for {:?} is {}",
    //     plum.plum_head.plum_body_seal,
    //     datahost
    //         .select_plum_body_reference_count(&plum.plum_head.plum_body_seal)
    //         .await
    //         .expect("pass")
    // );
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_create_plums_with_identical_bodies() {
    let string = format!(
        "test_datahost_create_plums_with_identical_bodies, {}",
        Uuid::new_v4()
    );
    let plum_0 = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &string,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let plum_1 = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &string,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let head_seal_0 = datahost.store_plum(&plum_0, None).await.expect("pass");
    assert_eq!(head_seal_0, PlumHeadSeal::from(&plum_0.plum_head));

    let head_seal_1 = datahost.store_plum(&plum_1, None).await.expect("pass");
    assert_eq!(head_seal_1, PlumHeadSeal::from(&plum_1.plum_head));

    // let _ = plum_body_seal;
    // let body_reference_count = datahost
    //     .select_plum_body_reference_count(&plum_body_seal)
    //     .await
    //     .expect("pass");
    // assert_eq!(body_reference_count, 2);

    // log::debug!(
    //     "reference count for {:?} is {}",
    //     plum_body_seal,
    //     body_reference_count
    // );
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_branch_node() {
    let datahost = datahost_from_env_var().await;

    let content_1 = format!("splunges are cool, {}", Uuid::new_v4());
    let content_2 = format!("HIPPOS are cool, {}", Uuid::new_v4());

    let content_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_1,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let content_2_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_2,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let metadata_0_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("Branch root, {}", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let metadata_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("Initial statement, {}", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let metadata_2_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!(
                "Revised statement authored by the HIPPO lobby, {}",
                Uuid::new_v4()
            ),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let content_1_plum_head_seal = datahost
        .store_plum(&content_1_plum, None)
        .await
        .expect("pass");
    let content_2_plum_head_seal = datahost
        .store_plum(&content_2_plum, None)
        .await
        .expect("pass");

    let metadata_0_plum_head_seal = datahost
        .store_plum(&metadata_0_plum, None)
        .await
        .expect("pass");
    let metadata_1_plum_head_seal = datahost
        .store_plum(&metadata_1_plum, None)
        .await
        .expect("pass");
    let metadata_2_plum_head_seal = datahost
        .store_plum(&metadata_2_plum, None)
        .await
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
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let branch_node_0_plum_head_seal = datahost
        .store_plum(&branch_node_0_plum, None)
        .await
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
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let branch_node_1_plum_head_seal = datahost
        .store_plum(&branch_node_1_plum, None)
        .await
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
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let branch_node_2_plum_head_seal = datahost
        .store_plum(&branch_node_2_plum, None)
        .await
        .expect("pass");

    //
    // Now accumulate_relations_recursive and check the results.  branch_node_2_plum is the head
    // of the branch, so it should depend on all other Plums.
    // TODO: Actually check the PlumRelationFlags values
    //

    {
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
        let plum_relation_flags_m = datahost
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
}

#[tokio::test]
#[serial_test::serial]
async fn test_datahost_dir_node() {
    let datahost = datahost_from_env_var().await;

    let content_0 = format!("ostriches are cool, {}", Uuid::new_v4());
    let content_1 = format!("splunges are cool, {}", Uuid::new_v4());

    let content_0_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_0,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let content_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_1,
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let content_0_plum_head_seal = datahost
        .store_plum(&content_0_plum, None)
        .await
        .expect("pass");
    let content_1_plum_head_seal = datahost
        .store_plum(&content_1_plum, None)
        .await
        .expect("pass");

    let dir_node_0 = DirNode {
        // Make this one an empty DirNode.
        entry_m: BTreeMap::new(),
    };
    let dir_node_0_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &dir_node_0,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_0_plum_head_seal = datahost
        .store_plum(&dir_node_0_plum, None)
        .await
        .expect("pass");

    let dir_node_1 = DirNode {
        entry_m: maplit::btreemap! {
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
        },
    };
    let dir_node_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &dir_node_1,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_1_plum_head_seal = datahost
        .store_plum(&dir_node_1_plum, None)
        .await
        .expect("pass");

    let dir_node_2 = DirNode {
        entry_m: maplit::btreemap! {
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_2_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &dir_node_2,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_2_plum_head_seal = datahost
        .store_plum(&dir_node_2_plum, None)
        .await
        .expect("pass");

    let dir_node_3 = DirNode {
        entry_m: maplit::btreemap! {
            "dir0".to_string() => dir_node_0_plum_head_seal.clone(),
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_3_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &dir_node_3,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_3_plum_head_seal = datahost
        .store_plum(&dir_node_3_plum, None)
        .await
        .expect("pass");

    let dir_node_4 = DirNode {
        entry_m: maplit::btreemap! {
            "dir1".to_string() => dir_node_1_plum_head_seal.clone(),
            "dir2".to_string() => dir_node_2_plum_head_seal.clone(),
        },
    };
    let dir_node_4_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &dir_node_4,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_4_plum_head_seal = datahost
        .store_plum(&dir_node_4_plum, None)
        .await
        .expect("pass");

    //
    // Now accumulate_relations_recursive and check the results.
    // TODO: Actually check the PlumRelationFlags values
    //

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&content_0_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }
    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&content_1_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&dir_node_0_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        assert_eq!(plum_relation_flags_m.len(), 0);
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&dir_node_1_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        // These are the dependencies of dir_node_1_plum
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&dir_node_2_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        // These are the dependencies of dir_node_2_plum
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&dir_node_3_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        // These are the dependencies of dir_node_3_plum.  Note that dir_node_0_plum contains no entries.
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&dir_node_0_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&dir_node_4_plum),
                PlumRelationFlags::ALL,
                None,
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        // These are the dependencies of dir_node_4_plum.  Note that content_0_plum and content_1_plum are
        // both contained in dir_node_2_plum, and content_0_plum is contained in dir_node_1_plum.
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&dir_node_1_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&dir_node_2_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(plum_relation_flags_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    //
    // Test FragmentQueryable
    //

    {
        assert_eq!(
            dir_node_0
                .fragment_query_single_segment(&dir_node_0_plum_head_seal, "")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        // This should be the same as querying for "" -- TODO FIXME seems to not be working
        assert_eq!(
            dir_node_0
                .fragment_query_single_segment(&dir_node_0_plum_head_seal, "/")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        assert!(dir_node_0
            .fragment_query_single_segment(&dir_node_0_plum_head_seal, "nonexistent")
            .is_err());

        assert_eq!(
            dir_node_1
                .fragment_query_single_segment(&dir_node_1_plum_head_seal, "")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_1_plum_head_seal.clone()),
        );
        assert!(dir_node_1
            .fragment_query_single_segment(&dir_node_1_plum_head_seal, "nonexistent")
            .is_err());
        assert_eq!(
            dir_node_1
                .fragment_query_single_segment(&dir_node_1_plum_head_seal, "ostriches.txt")
                .expect("pass"),
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );

        assert_eq!(
            dir_node_2
                .fragment_query_single_segment(&dir_node_2_plum_head_seal, "")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_2_plum_head_seal.clone()),
        );
        assert!(dir_node_2
            .fragment_query_single_segment(&dir_node_2_plum_head_seal, "nonexistent")
            .is_err());
        assert_eq!(
            dir_node_2
                .fragment_query_single_segment(&dir_node_2_plum_head_seal, "ostriches.txt")
                .expect("pass"),
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_2
                .fragment_query_single_segment(&dir_node_2_plum_head_seal, "splunges.txt")
                .expect("pass"),
            FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
        );

        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_3_plum_head_seal.clone()),
        );
        assert!(dir_node_3
            .fragment_query_single_segment(&dir_node_3_plum_head_seal, "nonexistent")
            .is_err());
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "ostriches.txt")
                .expect("pass"),
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "splunges.txt")
                .expect("pass"),
            FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0")
                .expect("pass"),
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/stuff")
                .expect("pass"),
            FragmentQueryResult::ForwardQueryTo {
                target: dir_node_0_plum_head_seal.clone(),
                rest_of_query_str: "stuff",
            },
        );
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/")
                .expect("pass"),
            FragmentQueryResult::ForwardQueryTo {
                target: dir_node_0_plum_head_seal.clone(),
                rest_of_query_str: "",
            },
        );
        assert_eq!(
            dir_node_3
                .fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/stuff/and/things")
                .expect("pass"),
            FragmentQueryResult::ForwardQueryTo {
                target: dir_node_0_plum_head_seal.clone(),
                rest_of_query_str: "stuff/and/things",
            },
        );
    }

    //
    // Datahost fragment_query
    //

    {
        assert_eq!(
            datahost
                .fragment_query(&dir_node_0_plum_head_seal, "", None)
                .await
                .expect("pass"),
            dir_node_0_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_0_plum_head_seal, "nonexistent", None)
            .await
            .is_err());

        assert_eq!(
            datahost
                .fragment_query(&dir_node_1_plum_head_seal, "", None)
                .await
                .expect("pass"),
            dir_node_1_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_1_plum_head_seal, "nonexistent", None)
            .await
            .is_err());
        assert_eq!(
            datahost
                .fragment_query(&dir_node_1_plum_head_seal, "ostriches.txt", None)
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost
                .fragment_query(&dir_node_3_plum_head_seal, "dir0", None)
                .await
                .expect("pass"),
            dir_node_0_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_3_plum_head_seal, "ostriches.txt", None)
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir1", None)
                .await
                .expect("pass"),
            dir_node_1_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2", None)
                .await
                .expect("pass"),
            dir_node_2_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir1/ostriches.txt", None)
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_4_plum_head_seal, "dir1/nonexistent", None)
            .await
            .is_err());
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2/ostriches.txt", None)
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2/splunges.txt", None)
                .await
                .expect("pass"),
            content_1_plum_head_seal,
        );
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_plum_ref() {
    let datahost = datahost_from_env_var().await;
    let datahost_la = Arc::new(RwLock::new(datahost));
    Datacache::set_singleton(Box::new(Datacache::new(datahost_la.clone())));

    let content_0 = format!("ostriches are cool, {}", Uuid::new_v4());
    let content_1 = format!("actually, HIPPOs are cool, {}", Uuid::new_v4()).into_bytes();

    let content_0_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_0,
            ContentFormat::msgpack(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let content_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_1,
            ContentFormat::msgpack(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let content_0_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(&content_0_plum, None)
        .await
        .expect("pass");
    let content_1_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(&content_1_plum, None)
        .await
        .expect("pass");

    let content_0_plum_uri = PlumURI::from(PlumURILocal::from(content_0_plum_head_seal.clone()));
    let content_1_plum_uri = PlumURI::from(PlumURILocal::from(content_1_plum_head_seal.clone()));

    let plum_0_ref = PlumRef::<String>::new(content_0_plum_uri);
    let plum_1_ref = PlumRef::<Vec<u8>>::new(content_1_plum_uri);

    log::debug!("plum_0_ref: {:?}", plum_0_ref);
    log::debug!("plum_1_ref: {:?}", plum_1_ref);

    assert!(!plum_0_ref.value_is_cached().await);
    assert!(!plum_1_ref.value_is_cached().await);

    log::debug!(
        "plum_0_ref typed content: {:?}",
        plum_0_ref.value_a().await.expect("pass")
    );
    log::debug!(
        "plum_1_ref typed content: {:?}",
        plum_1_ref.value_a().await.expect("pass")
    );
    assert_eq!(
        plum_0_ref
            .value_a()
            .await
            .expect("pass")
            // .as_ref()
            .as_str(),
        content_0.as_str()
    );
    assert_eq!(*plum_1_ref.value_a().await.expect("pass"), content_1);
    assert!(plum_0_ref.value_is_cached().await);
    assert!(plum_1_ref.value_is_cached().await);

    // Clear the cache and then try to access plum_0_ref's and plum_1_ref's values again
    Datacache::singleton().clear_cache().await;

    assert!(!plum_0_ref.value_is_cached().await);
    assert!(!plum_1_ref.value_is_cached().await);

    // log::debug!(
    //     "plum_0_ref typed content: {:?}",
    //     plum_0_ref.typed_content().expect("pass")
    // );
    // log::debug!(
    //     "plum_1_ref typed content: {:?}",
    //     plum_1_ref.typed_content().expect("pass")
    // );
    assert_eq!(
        plum_0_ref
            .value_a()
            .await
            .expect("pass")
            // .as_ref()
            .as_str(),
        content_0.as_str()
    );
    assert_eq!(*plum_1_ref.value_a().await.expect("pass"), content_1);
    assert!(plum_0_ref.value_is_cached().await);
    assert!(plum_1_ref.value_is_cached().await);
}

#[tokio::test]
#[serial_test::serial]
async fn test_path_state() {
    let datahost = datahost_from_env_var().await;

    let path = Path::from(format!("fancypath-{}", Uuid::new_v4()));

    let plum0 = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("HIPPOs and OSTRICHes are enemies! {}", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let plum0_head_seal = PlumHeadSeal::from(&plum0.plum_head);
    let plum1 = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!(
                "No, HIPPOs and OSTRICHes are friends forever. {}",
                Uuid::new_v4()
            ),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let plum1_head_seal = PlumHeadSeal::from(&plum1.plum_head);

    assert!(!datahost.has_path_state(&path, None).await.expect("pass"));

    datahost
        .insert_path_state(
            &PathState {
                path: path.clone(),
                current_state_plum_head_seal: plum0_head_seal.clone(),
            },
            None,
        )
        .await
        .expect("pass");

    assert!(datahost.has_path_state(&path, None).await.expect("pass"));
    {
        let path_state = datahost.load_path_state(&path, None).await.expect("pass");
        assert_eq!(path_state.path, path);
        assert_eq!(path_state.current_state_plum_head_seal, plum0_head_seal);
    }

    datahost
        .update_path_state(
            &PathState {
                path: path.clone(),
                current_state_plum_head_seal: plum1_head_seal.clone(),
            },
            None,
        )
        .await
        .expect("pass");

    assert!(datahost.has_path_state(&path, None).await.expect("pass"));
    {
        let path_state = datahost.load_path_state(&path, None).await.expect("pass");
        assert_eq!(path_state.path, path);
        assert_eq!(path_state.current_state_plum_head_seal, plum1_head_seal);
    }

    datahost.delete_path_state(&path, None).await.expect("pass");
    assert!(!datahost.has_path_state(&path, None).await.expect("pass"));
}

async fn build_and_store_random_branch_node_and_plum_with_ancestor(
    ancestor_o: Option<&Plum>,
    datahost: &Datahost,
) -> (BranchNode, Plum, PlumHeadSeal) {
    let metadata_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("BranchNode metadata {}", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let metadata_plum_head_seal = datahost
        .store_plum(&metadata_plum, None)
        .await
        .expect("pass");

    let content_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &format!("BranchNode content {}", Uuid::new_v4()),
            ContentFormat::charset_us_ascii(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let content_plum_head_seal = datahost
        .store_plum(&content_plum, None)
        .await
        .expect("pass");

    let branch_node = {
        let mut branch_node_builder = BranchNodeBuilder::new();
        branch_node_builder = if let Some(ancestor) = ancestor_o {
            branch_node_builder.with_ancestor(ancestor).expect("pass")
        } else {
            branch_node_builder
        };
        branch_node_builder
            .with_metadata(metadata_plum_head_seal)
            .with_content(content_plum_head_seal)
            .build()
            .expect("pass")
    };

    let branch_node_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &branch_node,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    let branch_node_plum_head_seal = datahost
        .store_plum(&branch_node_plum, None)
        .await
        .expect("pass");

    assert_eq!(
        branch_node_plum_head_seal,
        PlumHeadSeal::from(&branch_node_plum.plum_head)
    );

    (branch_node, branch_node_plum, branch_node_plum_head_seal)
}

#[tokio::test]
#[serial_test::serial]
async fn test_branch() {
    let datahost = datahost_from_env_var().await;

    let (_branch_node_0, branch_node_0_plum, branch_node_0_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(None, &datahost).await;
    let (_branch_node_1, branch_node_1_plum, branch_node_1_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_0_plum),
            &datahost,
        )
        .await;

    // Make a fork here into branches a and b.

    let (_branch_node_a2, branch_node_a2_plum, branch_node_a2_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_1_plum),
            &datahost,
        )
        .await;
    let (_branch_node_a3, branch_node_a3_plum, branch_node_a3_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_a2_plum),
            &datahost,
        )
        .await;
    let (_branch_node_a4, _branch_node_a4_plum, branch_node_a4_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_a3_plum),
            &datahost,
        )
        .await;

    let (_branch_node_b2, branch_node_b2_plum, branch_node_b2_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_1_plum),
            &datahost,
        )
        .await;
    let (_branch_node_b3, branch_node_b3_plum, branch_node_b3_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_b2_plum),
            &datahost,
        )
        .await;
    let (_branch_node_b4, _branch_node_b4_plum, branch_node_b4_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&branch_node_b3_plum),
            &datahost,
        )
        .await;

    // Create a totally unrelated branch as well, to test closest common ancestor.
    let (_other_branch_node_0, other_branch_node_0_plum, other_branch_node_0_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(None, &datahost).await;
    let (_other_branch_node_1, other_branch_node_1_plum, other_branch_node_1_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&other_branch_node_0_plum),
            &datahost,
        )
        .await;
    let (_other_branch_node_2, _other_branch_node_2_plum, other_branch_node_2_plum_head_seal) =
        build_and_store_random_branch_node_and_plum_with_ancestor(
            Some(&other_branch_node_1_plum),
            &datahost,
        )
        .await;

    // Test computation of closest common ancestor.
    {
        struct ClosestCommonAncestorTestCase<'a> {
            lhs: &'a PlumHeadSeal,
            rhs: &'a PlumHeadSeal,
            expect_o: Option<&'a PlumHeadSeal>,
        }

        for closest_common_ancestor_test_case in &[
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_0_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_1_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a2_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a3_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a4_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: Some(&branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_0_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_1_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a2_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a3_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a4_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_1_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a2_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_a2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a3_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_a2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a4_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_a2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_a2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a3_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: Some(&branch_node_a3_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a4_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: Some(&branch_node_a3_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_a3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_a4_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: Some(&branch_node_a4_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: Some(&branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_a4_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b2_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: Some(&branch_node_b2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: Some(&branch_node_b2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: Some(&branch_node_b2_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_b2_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b3_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: Some(&branch_node_b3_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: Some(&branch_node_b3_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &branch_node_b4_plum_head_seal,
                rhs: &branch_node_b4_plum_head_seal,
                expect_o: Some(&branch_node_b4_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &branch_node_b3_plum_head_seal,
                expect_o: None,
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_0_plum_head_seal,
                rhs: &other_branch_node_0_plum_head_seal,
                expect_o: Some(&other_branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &other_branch_node_0_plum_head_seal,
                expect_o: Some(&other_branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &other_branch_node_0_plum_head_seal,
                expect_o: Some(&other_branch_node_0_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_1_plum_head_seal,
                rhs: &other_branch_node_1_plum_head_seal,
                expect_o: Some(&other_branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &other_branch_node_1_plum_head_seal,
                expect_o: Some(&other_branch_node_1_plum_head_seal),
            },
            ClosestCommonAncestorTestCase {
                lhs: &other_branch_node_2_plum_head_seal,
                rhs: &other_branch_node_2_plum_head_seal,
                expect_o: Some(&other_branch_node_2_plum_head_seal),
            },
        ] {
            assert_eq!(
                datahost
                    .closest_common_branch_node_ancestor(
                        closest_common_ancestor_test_case.lhs,
                        closest_common_ancestor_test_case.rhs,
                        None
                    )
                    .await
                    .expect("pass"),
                closest_common_ancestor_test_case.expect_o.cloned(),
            );
            // Also test lhs <-> rhs, since it should be a symmetric relationship.
            assert_eq!(
                datahost
                    .closest_common_branch_node_ancestor(
                        closest_common_ancestor_test_case.rhs,
                        closest_common_ancestor_test_case.lhs,
                        None
                    )
                    .await
                    .expect("pass"),
                closest_common_ancestor_test_case.expect_o.cloned(),
            );
        }
    }

    let path = Path::from(format!("branchypath-{}", Uuid::new_v4()));

    assert!(!datahost.has_path_state(&path, None).await.expect("pass"));
    datahost
        .branch_get_head(&path, None)
        .await
        .expect_err("fail");

    // Verify that attempting to create a Branch without the BranchNode Plum causes an error.
    {
        // Create a Plum and don't store it, just use its PlumHeadSeal.
        let rando_plum = PlumBuilder::new()
            .with_plum_relations_and_plum_body_content_from(
                &format!("rando plum {}", Uuid::new_v4()),
                ContentFormat::charset_us_ascii(),
                ContentEncoding::none(),
            )
            .expect("pass")
            .build()
            .expect("pass");

        datahost
            .branch_create(
                &PathState {
                    path: path.clone(),
                    current_state_plum_head_seal: PlumHeadSeal::from(&rando_plum.plum_head),
                },
                None,
            )
            .await
            .expect_err("fail");
    }

    // Store the Plum, and try to create the branch again.
    let branch_node_0_plum_head_seal_ = datahost
        .store_plum(&branch_node_0_plum, None)
        .await
        .expect("pass");
    assert_eq!(branch_node_0_plum_head_seal_, branch_node_0_plum_head_seal);
    datahost
        .branch_create(
            &PathState {
                path: path.clone(),
                current_state_plum_head_seal: branch_node_0_plum_head_seal.clone(),
            },
            None,
        )
        .await
        .expect("pass");

    assert!(datahost.has_path_state(&path, None).await.expect("pass"));

    // Get the branch head and verify it's correct.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_0_plum_head_seal
    );

    // Now set the branch head to one of the forks -- fast forward.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchFastForwardTo(
                    branch_node_a4_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect("pass");
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_a4_plum_head_seal
    );

    // Verify that fast-forward to an ancestor fails.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchFastForwardTo(
                    branch_node_1_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_a4_plum_head_seal
    );

    // Verify that fast-forward to a no-common-ancestor node fails.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchFastForwardTo(
                    other_branch_node_0_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_a4_plum_head_seal
    );

    // Now set the branch head back to the common ancestor -- rewind.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchRewindTo(
                    branch_node_1_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect("pass");
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_1_plum_head_seal
    );

    // Verify that rewind to a descendant fails.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchRewindTo(
                    branch_node_a4_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_1_plum_head_seal
    );

    // Verify that rewind to a no-common-ancestor node fails.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchRewindTo(
                    other_branch_node_2_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_1_plum_head_seal
    );

    // Now set the branch head to the other of the forks -- fast forward.
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchFastForwardTo(
                    branch_node_b4_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect("pass");
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_b4_plum_head_seal
    );

    // Now set the branch head to the first of the forks again -- reset
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchForkHistoryTo(
                    branch_node_a4_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect("pass");
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_a4_plum_head_seal
    );

    // Verify that fork-history to a no-common-ancestor node fails
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchForkHistoryTo(
                    other_branch_node_2_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        branch_node_a4_plum_head_seal
    );

    // Test the case where there's no common ancestor (this would be where someone tries to change the branch
    // to one with no shared history at all, which should be a different operation entirely; similar to git filter)
    // Now set the branch head to the first of the forks again -- reset
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchTotallyRewriteTo(
                    other_branch_node_2_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect("pass");
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        other_branch_node_2_plum_head_seal
    );

    // Verify that total-rewrite to a common-ancestor node fails
    datahost
        .branch_set_head(
            BranchSetHeadRequest {
                branch_path: path.clone(),
                value: Some(branch_set_head_request::Value::BranchTotallyRewriteTo(
                    other_branch_node_1_plum_head_seal.clone(),
                )),
            },
            None,
        )
        .await
        .expect_err("fail");
    // Verify that the branch head didn't change.
    assert_eq!(
        datahost.branch_get_head(&path, None).await.expect("pass"),
        other_branch_node_2_plum_head_seal
    );

    // Verify that attempting to set the branch head to a non-BranchNode fails.
    {
        let non_branch_node_plum = PlumBuilder::new()
            .with_plum_relations_and_plum_body_content_from(
                &format!("a mad hippo is a glad hippo; {}", Uuid::new_v4()),
                ContentFormat::charset_us_ascii(),
                ContentEncoding::none(),
            )
            .expect("pass")
            .build()
            .expect("pass");
        let non_branch_node_plum_head_seal = datahost
            .store_plum(&non_branch_node_plum, None)
            .await
            .expect("pass");
        datahost
            .branch_set_head(
                BranchSetHeadRequest {
                    branch_path: path.clone(),
                    value: Some(branch_set_head_request::Value::BranchFastForwardTo(
                        non_branch_node_plum_head_seal.clone(),
                    )),
                },
                None,
            )
            .await
            .expect_err("fail");
        // TODO: Could test other branch_set_head_request::Value variants, but this is probably fine.
    }

    // Now delete the branch
    datahost.branch_delete(&path, None).await.expect("pass");
    datahost
        .branch_get_head(&path, None)
        .await
        .expect_err("fail");

    // Verify that deleting a non-existent branch fails.
    datahost
        .branch_delete(
            &Path::from(format!("nonexistent-branch-{}", Uuid::new_v4())),
            None,
        )
        .await
        .expect_err("fail");
}
