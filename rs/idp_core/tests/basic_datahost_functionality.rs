use idp_core::{BranchNode, Datahost, DirNode, FragmentQueryable, FragmentQueryResult, RelationFlags};
use idp_proto::{ContentType, Nonce, Plum, PlumBodyBuilder, PlumBodySeal, PlumBuilder, PlumHeadBuilder, PlumHeadSeal};
use serial_test::serial;
use std::collections::BTreeMap;
use uuid::Uuid;

#[test]
#[serial]
fn open_datahost() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    Datahost::open_using_env_var()?;
    Ok(())
}

#[test]
#[serial]
fn open_and_close_datahost() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let datahost = Datahost::open_using_env_var()?;
    datahost.close();
    Ok(())
}

#[test]
#[serial]
fn test_datahost_create_plum_head() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain".as_bytes()))
        .with_body_content(format!("test_datahost_create_plum_head, {}.", Uuid::new_v4()).as_bytes().to_vec())
        .build()?;

    let datahost = Datahost::open_using_env_var()?;

    let head_seal = datahost.create_plum_head(&plum.head)?;
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.head));

    // create_plum_head again and ensure it worked again.
    let head_seal_2 = datahost.create_plum_head(&plum.head)?;
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum.head));

    // read_plum_head and check that it matches.
    let plum_head_2 = datahost.read_plum_head(&head_seal)?;
    assert_eq!(plum_head_2, plum.head);

    log::debug!("reference count for {:?} is {}", plum.head.body_seal, datahost.select_plum_body_reference_count(&plum.head.body_seal)?);

    Ok(())
}

#[test]
#[serial]
fn test_datahost_create_plum_body() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let plum_body = PlumBodyBuilder::new()
        .with_body_content(format!("test_datahost_create_plum_body, {}.", Uuid::new_v4()).as_bytes().to_vec())
        .build()?;

    let datahost = Datahost::open_using_env_var()?;

    let body_seal = datahost.create_plum_body(&plum_body)?;
    assert_eq!(body_seal, PlumBodySeal::from(&plum_body));

    // create_plum_body again and ensure it worked again.
    let body_seal_2 = datahost.create_plum_body(&plum_body)?;
    assert_eq!(body_seal_2, PlumBodySeal::from(&plum_body));

    log::debug!("reference count for {:?} is {}", body_seal, datahost.select_plum_body_reference_count(&body_seal)?);

    Ok(())
}

#[test]
#[serial]
fn test_datahost_create_plum() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content(format!("test_datahost_create_plum, {}.", Uuid::new_v4()).as_bytes().to_vec())
        .build()?;

    let datahost = Datahost::open_using_env_var()?;

    let head_seal = datahost.create_plum(&plum)?;
    assert_eq!(head_seal, PlumHeadSeal::from(&plum.head));

    // create_plum again and ensure it worked again
    let head_seal_2 = datahost.create_plum(&plum)?;
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum.head));

    log::debug!("reference count for {:?} is {}", plum.head.body_seal, datahost.select_plum_body_reference_count(&plum.head.body_seal)?);

    Ok(())
}

#[test]
#[serial]
fn test_datahost_create_plums_with_identical_bodies() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let plum_body = PlumBodyBuilder::new()
        .with_body_content(format!("test_datahost_create_plums_with_identical_bodies, {}.", Uuid::new_v4()).as_bytes().to_vec())
        .build()?;
//     let plum_body = PlumBody {
//         body_nonce_o: None,
//         body_content: format!("test_datahost_create_plums_with_identical_bodies, {}.", Uuid::new_v4()).as_bytes().to_vec(),
//     };
    let plum_head_0 = PlumHeadBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_head_nonce(Nonce::from("blahdy blah"))
        .build_with_body(&plum_body)?;
    let plum_head_1 = PlumHeadBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_head_nonce(Nonce::from("NOTHING"))
        .build_with_body(&plum_body)?;

//     let plum_head_0 = PlumHead {
//         body_seal: PlumBodySeal::from(&plum_body),
//         body_length: plum_body.body_content.len() as u64,
//         body_content_type: ContentType::from("text/plain"),
//         head_nonce_o: Some(Nonce::from("blahdy blah")),
//         owner_did_o: None,
//         created_at_o: None,
//         metadata_o: None,
//     };
//     let plum_head_1 = PlumHead {
//         body_seal: PlumBodySeal::from(&plum_body),
//         body_length: plum_body.body_content.len() as u64,
//         body_content_type: ContentType::from("text/plain"),
//         head_nonce_o: Some(Nonce::from("NOTHING")),
//         owner_did_o: None,
//         created_at_o: None,
//         metadata_o: None,
//     };
    let body_seal = plum_head_0.body_seal.clone();
    let plum_0 = Plum {
        head: plum_head_0,
        body: plum_body.clone(),
    };
    let plum_1 = Plum {
        head: plum_head_1,
        body: plum_body,
    };

    let datahost = Datahost::open_using_env_var()?;

    let head_seal_0 = datahost.create_plum(&plum_0)?;
    assert_eq!(head_seal_0, PlumHeadSeal::from(&plum_0.head));

    let head_seal_1 = datahost.create_plum(&plum_1)?;
    assert_eq!(head_seal_1, PlumHeadSeal::from(&plum_1.head));

    let body_reference_count = datahost.select_plum_body_reference_count(&body_seal)?;
    assert_eq!(body_reference_count, 2);

    log::debug!("reference count for {:?} is {}", body_seal, body_reference_count);

    Ok(())
}

#[test]
#[serial]
fn test_datahost_branch_node() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let datahost = Datahost::open_using_env_var()?;

    let content_1 = "splunges are cool";
    let content_2 = "HIPPOS are cool";

    let content_1_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content(content_1.as_bytes().to_vec())
        .build()?;
    let content_2_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content(content_2.as_bytes().to_vec())
        .build()?;

    let metadata_0_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Branch root".as_bytes().to_vec())
        .build()?;
    let metadata_1_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Initial statement".as_bytes().to_vec())
        .build()?;
    let metadata_2_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Revised statement authored by the HIPPO lobby".as_bytes().to_vec())
        .build()?;

    let content_1_plum_head_seal = datahost.create_plum(&content_1_plum)?;
    let content_2_plum_head_seal = datahost.create_plum(&content_2_plum)?;

    let metadata_0_plum_head_seal = datahost.create_plum(&metadata_0_plum)?;
    let metadata_1_plum_head_seal = datahost.create_plum(&metadata_1_plum)?;
    let metadata_2_plum_head_seal = datahost.create_plum(&metadata_2_plum)?;

    let branch_node_0 = BranchNode {
        ancestor_o: None,
        metadata: metadata_0_plum_head_seal.clone(),
        content_o: None,
        posi_diff_o: None,
        nega_diff_o: None,
    };
    let branch_node_0_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(rmp_serde::to_vec(&branch_node_0)?)
        .build()?;
    let branch_node_0_plum_head_seal = datahost.create_plum(&branch_node_0_plum)?;

    let branch_node_1 = BranchNode {
        ancestor_o: Some(branch_node_0_plum_head_seal.clone()),
        metadata: metadata_1_plum_head_seal.clone(),
        content_o: Some(content_1_plum_head_seal.clone()),
        posi_diff_o: None,
        nega_diff_o: None,
    };
    let branch_node_1_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(rmp_serde::to_vec(&branch_node_1)?)
        .build()?;
    let branch_node_1_plum_head_seal = datahost.create_plum(&branch_node_1_plum)?;

    let branch_node_2 = BranchNode {
        ancestor_o: Some(branch_node_1_plum_head_seal.clone()),
        metadata: metadata_2_plum_head_seal.clone(),
        content_o: Some(content_2_plum_head_seal.clone()),
        posi_diff_o: None,
        nega_diff_o: None,
    };
    let branch_node_2_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(rmp_serde::to_vec(&branch_node_2)?)
        .build()?;
    let branch_node_2_plum_head_seal = datahost.create_plum(&branch_node_2_plum)?;

    //
    // Now accumulate_relations_recursive and check the results.  branch_node_2_plum is the head
    // of the branch, so it should depend on all other Plums.
    // TODO: Actually check the RelationFlags values
    //

    {
        let relation_m = datahost.accumulated_relations_recursive(&content_1_plum_head_seal, RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&content_2_plum_head_seal, RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&metadata_0_plum_head_seal, RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&metadata_1_plum_head_seal, RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&metadata_2_plum_head_seal, RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&branch_node_0_plum_head_seal, RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        assert_eq!(relation_m.len(), 1);
        assert!(relation_m.contains_key(&metadata_0_plum_head_seal));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&branch_node_0_plum_head_seal, RelationFlags::CONTENT_DEPENDENCY)?;
        log::debug!("relation_m: {:?}", relation_m);
        // Empty because metadata is METADATA_DEPENDENCY.
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&branch_node_1_plum_head_seal, RelationFlags::ALL)?;

        log::debug!("relation_m: {:?}", relation_m);
        assert_eq!(relation_m.len(), 4);
        // These are the dependencies of branch_node_0_plum
        assert!(relation_m.contains_key(&metadata_0_plum_head_seal));
        // These are the dependencies of branch_node_1_plum
        assert!(relation_m.contains_key(&branch_node_0_plum_head_seal));
        assert!(relation_m.contains_key(&metadata_1_plum_head_seal));
        assert!(relation_m.contains_key(&content_1_plum_head_seal));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&branch_node_2_plum_head_seal, RelationFlags::ALL)?;

        log::debug!("relation_m: {:?}", relation_m);
        assert_eq!(relation_m.len(), 7);
        // These are the dependencies of branch_node_0_plum
        assert!(relation_m.contains_key(&metadata_0_plum_head_seal));
        // These are the dependencies of branch_node_1_plum
        assert!(relation_m.contains_key(&branch_node_0_plum_head_seal));
        assert!(relation_m.contains_key(&metadata_1_plum_head_seal));
        assert!(relation_m.contains_key(&content_1_plum_head_seal));
        // These are the dependencies of branch_node_2_plum
        assert!(relation_m.contains_key(&branch_node_1_plum_head_seal));
        assert!(relation_m.contains_key(&metadata_2_plum_head_seal));
        assert!(relation_m.contains_key(&content_2_plum_head_seal));
    }

    //
    // Testing FragmentQueryable
    //

    {
        // Self-query
        assert_eq!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "")?,
            FragmentQueryResult::Value(branch_node_0_plum_head_seal.clone()),
        );
        // No ancestor
        assert!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "ancestor").is_err()
        );
        // No content
        assert!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "content").is_err()
        );
        // Invalid entry
        assert!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "nonexistent").is_err()
        );
        assert_eq!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata")?,
            FragmentQueryResult::Value(metadata_0_plum_head_seal.clone()),
        );
        assert_eq!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata/")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_0_plum_head_seal.clone(), rest_of_query_str: "" },
        );
        assert_eq!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata/stuff")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_0_plum_head_seal.clone(), rest_of_query_str: "stuff" },
        );
        assert_eq!(
            branch_node_0.fragment_query_single_segment(&branch_node_0_plum_head_seal, "metadata/stuff/and/things")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_0_plum_head_seal.clone(), rest_of_query_str: "stuff/and/things" },
        );
    }

    {
        // Self-query
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "")?,
            FragmentQueryResult::Value(branch_node_1_plum_head_seal.clone()),
        );
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "ancestor")?,
            FragmentQueryResult::Value(branch_node_0_plum_head_seal.clone()),
        );
        // No content
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "content")?,
            FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
        );
        // Invalid entry
        assert!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "nonexistent").is_err()
        );
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata")?,
            FragmentQueryResult::Value(metadata_1_plum_head_seal.clone()),
        );
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata/")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_1_plum_head_seal.clone(), rest_of_query_str: "" },
        );
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata/stuff")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_1_plum_head_seal.clone(), rest_of_query_str: "stuff" },
        );
        assert_eq!(
            branch_node_1.fragment_query_single_segment(&branch_node_1_plum_head_seal, "metadata/stuff/and/things")?,
            FragmentQueryResult::ForwardQueryTo { target: metadata_1_plum_head_seal.clone(), rest_of_query_str: "stuff/and/things" },
        );
    }

    Ok(())
}

#[test]
#[serial]
fn test_datahost_dir_node() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let datahost = Datahost::open_using_env_var()?;

    let content_0 = "ostriches are cool";
    let content_1 = "splunges are cool";

    let content_0_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content(content_0.as_bytes().to_vec())
        .build()?;
    let content_1_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content(content_1.as_bytes().to_vec())
        .build()?;

    let content_0_plum_head_seal = datahost.create_plum(&content_0_plum)?;
    let content_1_plum_head_seal = datahost.create_plum(&content_1_plum)?;

    let dir_node_0 = DirNode {
        // Make this one an empty DirNode.
        entry_m: BTreeMap::new(),
    };
    let dir_node_0_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::DirNode"))
        .with_body_content(rmp_serde::to_vec(&dir_node_0)?)
        .build()?;
    let dir_node_0_plum_head_seal = datahost.create_plum(&dir_node_0_plum)?;

    let dir_node_1 = DirNode {
        entry_m: maplit::btreemap!{
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
        },
    };
    let dir_node_1_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::DirNode"))
        .with_body_content(rmp_serde::to_vec(&dir_node_1)?)
        .build()?;
    let dir_node_1_plum_head_seal = datahost.create_plum(&dir_node_1_plum)?;

    let dir_node_2 = DirNode {
        entry_m: maplit::btreemap!{
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_2_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::DirNode"))
        .with_body_content(rmp_serde::to_vec(&dir_node_2)?)
        .build()?;
    let dir_node_2_plum_head_seal = datahost.create_plum(&dir_node_2_plum)?;

    let dir_node_3 = DirNode {
        entry_m: maplit::btreemap!{
            "dir0".to_string() => dir_node_0_plum_head_seal.clone(),
            "ostriches.txt".to_string() => content_0_plum_head_seal.clone(),
            "splunges.txt".to_string() => content_1_plum_head_seal.clone(),
        },
    };
    let dir_node_3_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::DirNode"))
        .with_body_content(rmp_serde::to_vec(&dir_node_3)?)
        .build()?;
    let dir_node_3_plum_head_seal = datahost.create_plum(&dir_node_3_plum)?;

    let dir_node_4 = DirNode {
        entry_m: maplit::btreemap!{
            "dir1".to_string() => dir_node_1_plum_head_seal.clone(),
            "dir2".to_string() => dir_node_2_plum_head_seal.clone(),
        },
    };
    let dir_node_4_plum = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::DirNode"))
        .with_body_content(rmp_serde::to_vec(&dir_node_4)?)
        .build()?;
    let dir_node_4_plum_head_seal = datahost.create_plum(&dir_node_4_plum)?;

    //
    // Now accumulate_relations_recursive and check the results.
    // TODO: Actually check the RelationFlags values
    //

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&content_0_plum), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&content_1_plum), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&dir_node_0_plum), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        assert_eq!(relation_m.len(), 0);
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&dir_node_1_plum), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        // These are the dependencies of dir_node_1_plum
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&dir_node_2_plum), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        // These are the dependencies of dir_node_2_plum
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&dir_node_3_plum), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        // These are the dependencies of dir_node_3_plum.  Note that dir_node_0_plum contains no entries.
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&dir_node_0_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&dir_node_4_plum), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        // These are the dependencies of dir_node_4_plum.  Note that content_0_plum and content_1_plum are
        // both contained in dir_node_2_plum, and content_0_plum is contained in dir_node_1_plum.
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&dir_node_1_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&dir_node_2_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_0_plum)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_1_plum)));
    }

    //
    // Test FragmentQueryable
    //

    {
        assert_eq!(
            dir_node_0.fragment_query_single_segment(&dir_node_0_plum_head_seal, "")?,
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        // This should be the same as querying for ""
        assert_eq!(
            dir_node_0.fragment_query_single_segment(&dir_node_0_plum_head_seal, "/")?,
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        assert!(dir_node_0.fragment_query_single_segment(&dir_node_0_plum_head_seal, "nonexistent").is_err());

        assert_eq!(
            dir_node_1.fragment_query_single_segment(&dir_node_1_plum_head_seal, "")?,
            FragmentQueryResult::Value(dir_node_1_plum_head_seal.clone()),
        );
        assert!(dir_node_1.fragment_query_single_segment(&dir_node_1_plum_head_seal, "nonexistent").is_err());
        assert_eq!(
            dir_node_1.fragment_query_single_segment(&dir_node_1_plum_head_seal, "ostriches.txt")?,
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );

        assert_eq!(
            dir_node_2.fragment_query_single_segment(&dir_node_2_plum_head_seal, "")?,
            FragmentQueryResult::Value(dir_node_2_plum_head_seal.clone()),
        );
        assert!(dir_node_2.fragment_query_single_segment(&dir_node_2_plum_head_seal, "nonexistent").is_err());
        assert_eq!(
            dir_node_2.fragment_query_single_segment(&dir_node_2_plum_head_seal, "ostriches.txt")?,
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_2.fragment_query_single_segment(&dir_node_2_plum_head_seal, "splunges.txt")?,
            FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
        );

        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "")?,
            FragmentQueryResult::Value(dir_node_3_plum_head_seal.clone()),
        );
        assert!(dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "nonexistent").is_err());
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "ostriches.txt")?,
            FragmentQueryResult::Value(content_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "splunges.txt")?,
            FragmentQueryResult::Value(content_1_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0")?,
            FragmentQueryResult::Value(dir_node_0_plum_head_seal.clone()),
        );
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/stuff")?,
            FragmentQueryResult::ForwardQueryTo {
                target: dir_node_0_plum_head_seal.clone(),
                rest_of_query_str: "stuff",
            },
        );
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/")?,
            FragmentQueryResult::ForwardQueryTo {
                target: dir_node_0_plum_head_seal.clone(),
                rest_of_query_str: "",
            },
        );
        assert_eq!(
            dir_node_3.fragment_query_single_segment(&dir_node_3_plum_head_seal, "dir0/stuff/and/things")?,
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
            datahost.fragment_query(&dir_node_0_plum_head_seal, "")?,
            dir_node_0_plum_head_seal,
        );
        assert!(
            datahost.fragment_query(&dir_node_0_plum_head_seal, "nonexistent").is_err()
        );

        assert_eq!(
            datahost.fragment_query(&dir_node_1_plum_head_seal, "")?,
            dir_node_1_plum_head_seal,
        );
        assert!(
            datahost.fragment_query(&dir_node_1_plum_head_seal, "nonexistent").is_err()
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_1_plum_head_seal, "ostriches.txt")?,
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost.fragment_query(&dir_node_3_plum_head_seal, "dir0")?,
            dir_node_0_plum_head_seal,
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_3_plum_head_seal, "ostriches.txt")?,
            content_0_plum_head_seal,
        );

        assert_eq!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir1")?,
            dir_node_1_plum_head_seal,
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir2")?,
            dir_node_2_plum_head_seal,
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir1/ostriches.txt")?,
            content_0_plum_head_seal,
        );
        assert!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir1/nonexistent").is_err()
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir2/ostriches.txt")?,
            content_0_plum_head_seal,
        );
        assert_eq!(
            datahost.fragment_query(&dir_node_4_plum_head_seal, "dir2/splunges.txt")?,
            content_1_plum_head_seal,
        );
    }

    Ok(())
}

