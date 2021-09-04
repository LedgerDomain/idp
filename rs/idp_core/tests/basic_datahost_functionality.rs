use idp_core::{BranchNode, Datahost, RelationFlags};
use idp_proto::{ContentType, Nonce, Plum, PlumBodyBuilder, PlumBodySeal, PlumBuilder, PlumHeadBuilder, PlumHeadSeal};
use serial_test::serial;
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
fn test_datahost_create_branch_nodes() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let content_plum_1 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("splunges are cool".as_bytes().to_vec())
        .build()?;
    let content_plum_2 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("HIPPOS are cool".as_bytes().to_vec())
        .build()?;
    let content_plum_3 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("nothing is cool at all".as_bytes().to_vec())
        .build()?;

    let metadata_plum_0 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Branch root".as_bytes().to_vec())
        .build()?;
    let metadata_plum_1 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Initial statement".as_bytes().to_vec())
        .build()?;
    let metadata_plum_2 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("Revised statement authored by the HIPPO lobby".as_bytes().to_vec())
        .build()?;
    let metadata_plum_3 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("text/plain"))
        .with_body_content("I hate smurfberries".as_bytes().to_vec())
        .build()?;

    let branch_node_plum_0 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(
            rmp_serde::to_vec(
                &BranchNode {
                    ancestor_o: None,
                    metadata: PlumHeadSeal::from(&metadata_plum_0.head),
                    content_o: None,
                    posi_diff_o: None,
                    nega_diff_o: None,
                }
            )?
        )
        .build()?;
    let branch_node_plum_1 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(
            rmp_serde::to_vec(
                &BranchNode {
                    ancestor_o: Some(PlumHeadSeal::from(&branch_node_plum_0.head)),
                    metadata: PlumHeadSeal::from(&metadata_plum_1.head),
                    content_o: Some(PlumHeadSeal::from(&content_plum_1.head)),
                    posi_diff_o: None,
                    nega_diff_o: None,
                }
            )?
        )
        .build()?;
    let branch_node_plum_2 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(
            rmp_serde::to_vec(
                &BranchNode {
                    ancestor_o: Some(PlumHeadSeal::from(&branch_node_plum_1.head)),
                    metadata: PlumHeadSeal::from(&metadata_plum_2.head),
                    content_o: Some(PlumHeadSeal::from(&content_plum_2.head)),
                    posi_diff_o: None,
                    nega_diff_o: None,
                }
            )?
        )
        .build()?;
    let branch_node_plum_3 = PlumBuilder::new()
        .with_body_content_type(ContentType::from("idp::BranchNode"))
        .with_body_content(
            rmp_serde::to_vec(
                &BranchNode {
                    ancestor_o: Some(PlumHeadSeal::from(&branch_node_plum_2.head)),
                    metadata: PlumHeadSeal::from(&metadata_plum_3.head),
                    content_o: Some(PlumHeadSeal::from(&content_plum_3.head)),
                    posi_diff_o: None,
                    nega_diff_o: None,
                }
            )?
        )
        .build()?;

    let datahost = Datahost::open_using_env_var()?;

    //
    // Now add all the Plum-s to the Datahost.
    //

    datahost.create_plum(&content_plum_1)?;
    datahost.create_plum(&content_plum_2)?;
    datahost.create_plum(&content_plum_3)?;

    datahost.create_plum(&metadata_plum_0)?;
    datahost.create_plum(&metadata_plum_1)?;
    datahost.create_plum(&metadata_plum_2)?;
    datahost.create_plum(&metadata_plum_3)?;

    datahost.create_plum(&branch_node_plum_0)?;
    datahost.create_plum(&branch_node_plum_1)?;
    datahost.create_plum(&branch_node_plum_2)?;
    datahost.create_plum(&branch_node_plum_3)?;

    //
    // Now accumulate_relations_recursive and check the results.  branch_node_plum_3 is the head
    // of the branch, so it should depend on all other Plums.
    //

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&content_plum_1.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&content_plum_2.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&content_plum_3.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&metadata_plum_0.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&metadata_plum_1.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&metadata_plum_2.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }
    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&metadata_plum_3.head), RelationFlags::ALL)?;
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&branch_node_plum_0.head), RelationFlags::ALL)?;
        log::debug!("relation_m: {:?}", relation_m);
        assert_eq!(relation_m.len(), 1);
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&metadata_plum_0.head)));
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&branch_node_plum_0.head), RelationFlags::CONTENT_DEPENDENCY)?;
        log::debug!("relation_m: {:?}", relation_m);
        // Empty because metadata is METADATA_DEPENDENCY.
        assert!(relation_m.is_empty());
    }

    {
        let relation_m = datahost.accumulated_relations_recursive(&PlumHeadSeal::from(&branch_node_plum_1.head), RelationFlags::ALL)?;
        log::debug!("branch_node_plum_0 -> {:?}", PlumHeadSeal::from(&branch_node_plum_0.head));
        log::debug!("metadata_plum_0 -> {:?}", PlumHeadSeal::from(&metadata_plum_0.head));
        log::debug!("metadata_plum_1 -> {:?}", PlumHeadSeal::from(&metadata_plum_1.head));

        log::debug!("relation_m: {:?}", relation_m);
//         assert_eq!(relation_m.len(), 1);
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&metadata_plum_0.head)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&metadata_plum_1.head)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&branch_node_plum_0.head)));
        assert!(relation_m.contains_key(&PlumHeadSeal::from(&content_plum_1.head)));
    }

    Ok(())
}
