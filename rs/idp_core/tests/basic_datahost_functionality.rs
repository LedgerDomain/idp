use serial_test::serial;
use idp_core::Datahost;
use idp_proto::{ContentType, Nonce, Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal};
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

    let plum_body = PlumBody {
        body_nonce_o: None,
        body_content: format!("test_datahost_create_plum_head, {}.", Uuid::new_v4()).as_bytes().to_vec(),
    };
    let plum_head = PlumHead {
        body_seal: PlumBodySeal::from(&plum_body),
        body_length: plum_body.body_content.len() as i64,
        body_content_type: ContentType::from("text/plain".as_bytes()),
        head_nonce_o: None,
        owner_did_o: None,
        created_at_o: None,
        metadata_o: None,
    };

    let datahost = Datahost::open_using_env_var()?;

    let head_seal = datahost.create_plum_head(&plum_head)?;
    assert_eq!(head_seal, PlumHeadSeal::from(&plum_head));

    // create_plum_head again and ensure it worked again.
    let head_seal_2 = datahost.create_plum_head(&plum_head)?;
    assert_eq!(head_seal_2, PlumHeadSeal::from(&plum_head));

    // read_plum_head and check that it matches.
    let plum_head_2 = datahost.read_plum_head(&head_seal)?;
    assert_eq!(plum_head_2, plum_head);

    log::debug!("reference count for {:?} is {}", plum_head.body_seal, datahost.select_plum_body_reference_count(&plum_head.body_seal)?);

    Ok(())
}

#[test]
#[serial]
fn test_datahost_create_plum_body() -> Result<(), failure::Error> {
    let _ = env_logger::try_init();

    let plum_body = PlumBody {
        body_nonce_o: None,
        body_content: format!("test_datahost_create_plum_body, {}.", Uuid::new_v4()).as_bytes().to_vec(),
    };

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

    let plum_body = PlumBody {
        body_nonce_o: None,
        body_content: format!("test_datahost_create_plum, {}.", Uuid::new_v4()).as_bytes().to_vec(),
    };
    let plum_head = PlumHead {
        body_seal: PlumBodySeal::from(&plum_body),
        body_length: plum_body.body_content.len() as i64,
        body_content_type: ContentType::from("text/plain"),
        head_nonce_o: None,
        owner_did_o: None,
        created_at_o: None,
        metadata_o: None,
    };
    let plum = Plum {
        head: plum_head,
        body: plum_body,
    };

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

    let plum_body = PlumBody {
        body_nonce_o: None,
        body_content: format!("test_datahost_create_plums_with_identical_bodies, {}.", Uuid::new_v4()).as_bytes().to_vec(),
    };
    let plum_head_0 = PlumHead {
        body_seal: PlumBodySeal::from(&plum_body),
        body_length: plum_body.body_content.len() as i64,
        body_content_type: ContentType::from("text/plain"),
        head_nonce_o: Some(Nonce::from("blahdy blah")),
        owner_did_o: None,
        created_at_o: None,
        metadata_o: None,
    };
    let plum_head_1 = PlumHead {
        body_seal: PlumBodySeal::from(&plum_body),
        body_length: plum_body.body_content.len() as i64,
        body_content_type: ContentType::from("text/plain"),
        head_nonce_o: Some(Nonce::from("NOTHING")),
        owner_did_o: None,
        created_at_o: None,
        metadata_o: None,
    };
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
