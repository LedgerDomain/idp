use async_lock::RwLock;
use idp_core::{
    BranchNode, Datacache, Datahost, DirNode, FragmentQueryResult, FragmentQueryable, PlumRef,
    PlumURI, PlumURILocal,
};
use idp_datahost_storage_sqlite::DatahostStorageSQLite;
use idp_proto::{
    ContentType, Nonce, Plum, PlumBodyBuilder, PlumBodySeal, PlumBuilder, PlumHeadBuilder,
    PlumHeadSeal, PlumRelationFlags, Sha256Sum,
};
use serial_test::serial;
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
        "test".to_string(),
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
#[serial]
async fn open_datahost() {
    datahost_from_env_var().await;
}

#[tokio::test]
#[serial]
async fn test_datahost_create_plum_head() {
    let plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(
            format!("test_datahost_create_plum_head, {}.", Uuid::new_v4())
                .as_bytes()
                .to_vec(),
        )
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let head_seal = datahost
        .store_plum_head(&plum.plum_head)
        .await
        .expect("pass");
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.plum_head));

    // store_plum_head again and ensure it worked again.
    let head_seal_2 = datahost
        .store_plum_head(&plum.plum_head)
        .await
        .expect("pass");
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum.plum_head));

    // load_plum_head and check that it matches.
    let plum_head_2 = datahost.load_plum_head(&head_seal).await.expect("pass");
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
#[serial]
async fn test_datahost_create_plum_body() {
    let plum_body = PlumBodyBuilder::new()
        .with_plum_body_content(
            format!("test_datahost_create_plum_body, {}.", Uuid::new_v4())
                .as_bytes()
                .to_vec(),
        )
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let plum_body_seal = datahost.store_plum_body(&plum_body).await.expect("pass");
    assert_eq!(plum_body_seal, PlumBodySeal::from(&plum_body));

    // store_plum_body again and ensure it worked again.
    let body_seal_2 = datahost.store_plum_body(&plum_body).await.expect("pass");
    assert_eq!(body_seal_2, PlumBodySeal::from(&plum_body));

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
#[serial]
async fn test_datahost_create_plum() {
    let plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(
            format!("test_datahost_create_plum, {}.", Uuid::new_v4())
                .as_bytes()
                .to_vec(),
        )
        .build()
        .expect("pass");

    let datahost = datahost_from_env_var().await;

    let head_seal = datahost.store_plum(&plum).await.expect("pass");
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.plum_head));

    // store_plum again and ensure it worked again
    let head_seal_2 = datahost.store_plum(&plum).await.expect("pass");
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
#[serial]
async fn test_datahost_create_plums_with_identical_bodies() {
    let plum_body = PlumBodyBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(
            format!(
                "test_datahost_create_plums_with_identical_bodies, {}.",
                Uuid::new_v4()
            )
            .as_bytes()
            .to_vec(),
        )
        .build()
        .expect("pass");
    let plum_body_seal = PlumBodySeal::from(&plum_body);
    let plum_head_0 = PlumHeadBuilder::new()
        .with_plum_head_nonce(Nonce::from("blahdy blah".as_bytes().to_vec()))
        .with_plum_body_seal(plum_body_seal.clone())
        .build()
        .expect("pass");
    let plum_head_1 = PlumHeadBuilder::new()
        .with_plum_head_nonce(Nonce::from("NOTHING".as_bytes().to_vec()))
        .with_plum_body_seal(plum_body_seal.clone())
        .build()
        .expect("pass");

    let plum_body_seal = plum_head_0.plum_body_seal.clone();
    let plum_0 = Plum {
        plum_head: plum_head_0,
        plum_relations_o: None,
        plum_body: plum_body.clone(),
    };
    let plum_1 = Plum {
        plum_head: plum_head_1,
        plum_relations_o: None,
        plum_body,
    };

    let datahost = datahost_from_env_var().await;

    let head_seal_0 = datahost.store_plum(&plum_0).await.expect("pass");
    assert_eq!(head_seal_0, PlumHeadSeal::from(&plum_0.plum_head));

    let head_seal_1 = datahost.store_plum(&plum_1).await.expect("pass");
    assert_eq!(head_seal_1, PlumHeadSeal::from(&plum_1.plum_head));

    let _ = plum_body_seal;
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
#[serial]
async fn test_datahost_branch_node() {
    let datahost = datahost_from_env_var().await;

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

    let content_1_plum_head_seal = datahost.store_plum(&content_1_plum).await.expect("pass");
    let content_2_plum_head_seal = datahost.store_plum(&content_2_plum).await.expect("pass");

    let metadata_0_plum_head_seal = datahost.store_plum(&metadata_0_plum).await.expect("pass");
    let metadata_1_plum_head_seal = datahost.store_plum(&metadata_1_plum).await.expect("pass");
    let metadata_2_plum_head_seal = datahost.store_plum(&metadata_2_plum).await.expect("pass");

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
    let branch_node_0_plum_head_seal = datahost
        .store_plum(&branch_node_0_plum)
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
    let branch_node_1_plum_head_seal = datahost
        .store_plum(&branch_node_1_plum)
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
    let branch_node_2_plum_head_seal = datahost
        .store_plum(&branch_node_2_plum)
        .await
        .expect("pass");

    //
    // Now accumulate_relations_recursive and check the results.  branch_node_2_plum is the head
    // of the branch, so it should depend on all other Plums.
    // TODO: Actually check the PlumRelationFlags values
    //

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&content_1_plum_head_seal, PlumRelationFlags::ALL)
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }
    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&content_2_plum_head_seal, PlumRelationFlags::ALL)
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&metadata_0_plum_head_seal, PlumRelationFlags::ALL)
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }
    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&metadata_1_plum_head_seal, PlumRelationFlags::ALL)
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }
    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&metadata_2_plum_head_seal, PlumRelationFlags::ALL)
            .await
            .expect("pass");
        assert!(plum_relation_flags_m.is_empty());
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&branch_node_0_plum_head_seal, PlumRelationFlags::ALL)
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
            )
            .await
            .expect("pass");
        log::debug!("plum_relation_flags_m: {:?}", plum_relation_flags_m);
        // Empty because metadata is METADATA_DEPENDENCY.
        assert!(plum_relation_flags_m.is_empty());
    }

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(&branch_node_1_plum_head_seal, PlumRelationFlags::ALL)
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
            .accumulated_relations_recursive(&branch_node_2_plum_head_seal, PlumRelationFlags::ALL)
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
#[serial]
async fn test_datahost_dir_node() {
    let datahost = datahost_from_env_var().await;

    let content_0 = format!("ostriches are cool, {}", Uuid::new_v4());
    let content_1 = format!("splunges are cool, {}", Uuid::new_v4());

    let content_0_plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(content_0.as_bytes().to_vec())
        .build()
        .expect("pass");
    let content_1_plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(content_1.as_bytes().to_vec())
        .build()
        .expect("pass");

    let content_0_plum_head_seal = datahost.store_plum(&content_0_plum).await.expect("pass");
    let content_1_plum_head_seal = datahost.store_plum(&content_1_plum).await.expect("pass");

    let dir_node_0 = DirNode {
        // Make this one an empty DirNode.
        entry_m: BTreeMap::new(),
    };
    let dir_node_0_plum = PlumBuilder::new()
        .with_relational_typed_content_from(&dir_node_0)
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_0_plum_head_seal = datahost.store_plum(&dir_node_0_plum).await.expect("pass");

    let dir_node_1 = DirNode {
        entry_m: maplit::btreemap! {
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
        },
    };
    let dir_node_1_plum = PlumBuilder::new()
        .with_relational_typed_content_from(&dir_node_1)
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_1_plum_head_seal = datahost.store_plum(&dir_node_1_plum).await.expect("pass");

    let dir_node_2 = DirNode {
        entry_m: maplit::btreemap! {
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_2_plum = PlumBuilder::new()
        .with_relational_typed_content_from(&dir_node_2)
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_2_plum_head_seal = datahost.store_plum(&dir_node_2_plum).await.expect("pass");

    let dir_node_3 = DirNode {
        entry_m: maplit::btreemap! {
            "dir0".to_string() => dir_node_0_plum_head_seal.clone(),
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_3_plum = PlumBuilder::new()
        .with_relational_typed_content_from(&dir_node_3)
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_3_plum_head_seal = datahost.store_plum(&dir_node_3_plum).await.expect("pass");

    let dir_node_4 = DirNode {
        entry_m: maplit::btreemap! {
            "dir1".to_string() => dir_node_1_plum_head_seal.clone(),
            "dir2".to_string() => dir_node_2_plum_head_seal.clone(),
        },
    };
    let dir_node_4_plum = PlumBuilder::new()
        .with_relational_typed_content_from(&dir_node_4)
        .expect("pass")
        .build()
        .expect("pass");
    let dir_node_4_plum_head_seal = datahost.store_plum(&dir_node_4_plum).await.expect("pass");

    //
    // Now accumulate_relations_recursive and check the results.
    // TODO: Actually check the PlumRelationFlags values
    //

    {
        let plum_relation_flags_m = datahost
            .accumulated_relations_recursive(
                &PlumHeadSeal::from(&content_0_plum),
                PlumRelationFlags::ALL,
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
                .fragment_query(&dir_node_0_plum_head_seal, "")
                .await
                .expect("pass"),
            dir_node_0_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_0_plum_head_seal, "nonexistent")
            .await
            .is_err());

        assert_eq!(
            datahost
                .fragment_query(&dir_node_1_plum_head_seal, "")
                .await
                .expect("pass"),
            dir_node_1_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_1_plum_head_seal, "nonexistent")
            .await
            .is_err());
        assert_eq!(
            datahost
                .fragment_query(&dir_node_1_plum_head_seal, "ostriches.txt")
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost
                .fragment_query(&dir_node_3_plum_head_seal, "dir0")
                .await
                .expect("pass"),
            dir_node_0_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_3_plum_head_seal, "ostriches.txt")
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir1")
                .await
                .expect("pass"),
            dir_node_1_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2")
                .await
                .expect("pass"),
            dir_node_2_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir1/ostriches.txt")
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );
        assert!(datahost
            .fragment_query(&dir_node_4_plum_head_seal, "dir1/nonexistent")
            .await
            .is_err());
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2/ostriches.txt")
                .await
                .expect("pass"),
            content_0_plum_head_seal,
        );
        assert_eq!(
            datahost
                .fragment_query(&dir_node_4_plum_head_seal, "dir2/splunges.txt")
                .await
                .expect("pass"),
            content_1_plum_head_seal,
        );
    }
}

#[tokio::test]
#[serial]
async fn test_plum_ref() {
    let datahost = datahost_from_env_var().await;
    let datahost_la = Arc::new(RwLock::new(datahost));
    Datacache::set_singleton(Box::new(Datacache::new(datahost_la.clone())));

    let content_0 = format!("ostriches are cool, {}", Uuid::new_v4());
    let content_1 = 12345678u32;

    let content_0_body_content = rmp_serde::to_vec(&content_0).expect("pass");
    let content_1_body_content = rmp_serde::to_vec(&content_1).expect("pass");

    let content_0_plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(content_0_body_content)
        .build()
        .expect("pass");
    let content_1_plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(content_1_body_content)
        .build()
        .expect("pass");

    let content_0_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(&content_0_plum)
        .await
        .expect("pass");
    let content_1_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(&content_1_plum)
        .await
        .expect("pass");

    let content_0_plum_uri = PlumURI::from(PlumURILocal::from(content_0_plum_head_seal.clone()));
    let content_1_plum_uri = PlumURI::from(PlumURILocal::from(content_1_plum_head_seal.clone()));

    let plum_0_ref = PlumRef::<String>::new(content_0_plum_uri);
    let plum_1_ref = PlumRef::<u32>::new(content_1_plum_uri);

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
