// TEMP HACK
#![allow(unused)]

use idp_proto::{
    serialize_and_encode_to_content, Content, ContentClass, ContentClassifiable, ContentEncoding,
    ContentFormat, ContentMetadata, ContentType, Contentifiable, Nonce, Plum, PlumBodySeal,
    PlumBuilder, PlumHeadSeal, PlumMetadata, PlumRelationFlags, PlumRelationFlagsMapping,
    PlumRelational, PlumRelationsBuilder, UnixNanoseconds,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// This will run once at load time (i.e. presumably before main function is called).
#[ctor::ctor]
fn overall_init() {
    env_logger::init();
}

#[test]
fn test_unix_nanoseconds_now_roundtrip() {
    let now = UnixNanoseconds::now();
    let chrono_now: chrono::DateTime<chrono::Utc> = now.into();
    let roundtrip_now: UnixNanoseconds = chrono_now.into();
    assert_eq!(roundtrip_now, now);
}

// Dummy type for use as a typed body.
#[derive(Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct DummyData {
    pub name: String,
    pub dependency: PlumHeadSeal,
}

impl ContentClassifiable for DummyData {
    fn content_class_str() -> &'static str {
        "application/x.idp.tests.DummyTypedBody"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}

impl Contentifiable for DummyData {
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match content_format.as_str() {
            // NOTE: There would be a cfg feature gate on this if this code were moved into the idp_proto crate.
            "json" => {
                serde_json::to_writer(writer, self)?;
            }
            // NOTE: There would be a cfg feature gate on this if this code were moved into the idp_proto crate.
            "msgpack" => {
                rmp_serde::encode::write(writer, self)?;
            }
            _ => {
                // NOTE: If you hit this, you may have just forgotten to enable one of the format-* features.
                anyhow::bail!("Unsupported ContentFormat: {:?}", content_format);
            }
        }
        Ok(())
    }
}

impl PlumRelational for DummyData {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        plum_relation_flags_m.insert(
            self.dependency.clone(),
            PlumRelationFlags::CONTENT_DEPENDENCY,
        );
    }
}

#[test]
fn test_gzip() {
    let data = format!("blahhhhh {} thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy thingy", Uuid::new_v4()).into_bytes();
    log::debug!("data (len {}): {:?}", data.len(), data);
    let mut compressed = Vec::new();
    {
        let mut encoder = libflate::gzip::Encoder::new(&mut compressed).unwrap();
        use std::io::Write;
        encoder.write_all(&data).unwrap();
        encoder.finish().into_result().unwrap();
    }
    log::debug!("compressed (len {}): {:?}", compressed.len(), compressed);
    let mut decompressed = Vec::new();
    {
        let mut decoder = libflate::gzip::Decoder::new(&compressed[..]).unwrap();
        use std::io::Read;
        decoder.read_to_end(&mut decompressed).unwrap();
    }
    log::debug!(
        "decompressed (len {}): {:?}",
        decompressed.len(),
        decompressed
    );
    assert_eq!(decompressed, data);
}

#[test]
fn test_contentifiable_1() {
    let dummy_data = DummyData {
        name: "test_contentifiable_1_name".to_string(),
        // Just a made-up PlumHeadSeal.
        dependency: PlumHeadSeal {
            value: idp_proto::Seal {
                sha256sum: idp_proto::Sha256Sum {
                    value: vec![234, 145, 182, 104],
                },
            },
        },
    };

    for content_encoding in [
        ContentEncoding::deflate(),
        ContentEncoding::gzip(),
        ContentEncoding::identity(),
    ]
    .iter()
    {
        let content = idp_proto::serialize_and_encode_to_content(
            &dummy_data,
            ContentFormat::json(),
            content_encoding.clone(),
        )
        .expect("pass");

        // Manually decode and deserialize, and check equality.
        let deserialized: DummyData = match content_encoding.as_str() {
            "deflate" => {
                let decoder = libflate::deflate::Decoder::new(content.content_byte_v.as_slice());
                serde_json::from_reader(decoder).expect("pass")
            }
            "gzip" => {
                let decoder =
                    libflate::gzip::Decoder::new(content.content_byte_v.as_slice()).expect("pass");
                serde_json::from_reader(decoder).expect("pass")
            }
            "identity" => serde_json::from_reader(content.content_byte_v.as_slice()).expect("pass"),
            _ => {
                panic!(
                    "Unsupported ContentEncoding: {:?}",
                    content_encoding.as_str()
                );
            }
        };
        assert_eq!(deserialized, dummy_data);
    }
}

#[test]
fn test_contentifiable_2() {
    let dummy_data = DummyData {
        name: "test_contentifiable_2_name".to_string(),
        // Just a made-up PlumHeadSeal.
        dependency: PlumHeadSeal {
            value: idp_proto::Seal {
                sha256sum: idp_proto::Sha256Sum {
                    value: vec![123, 45, 82, 24],
                },
            },
        },
    };

    for content_format in [ContentFormat::json(), ContentFormat::msgpack()] {
        for content_encoding in [
            ContentEncoding::none(),
            ContentEncoding::from(" ".to_string()),
            ContentEncoding::from("\t".to_string()),
            ContentEncoding::deflate(),
            ContentEncoding::gzip(),
            ContentEncoding::identity(),
            // 2 encoders in a row.
            ContentEncoding::from("identity,deflate".to_string()),
            ContentEncoding::from("identity, deflate".to_string()),
            ContentEncoding::from("identity , deflate".to_string()),
            ContentEncoding::from("identity ,\tdeflate".to_string()),
            ContentEncoding::from(" identity , deflate   ".to_string()),
            ContentEncoding::from("deflate,identity".to_string()),
            ContentEncoding::from("identity,gzip".to_string()),
            ContentEncoding::from("gzip,identity".to_string()),
            ContentEncoding::from("deflate,gzip".to_string()),
            ContentEncoding::from("gzip,deflate".to_string()),
            // Why the hell not...
            ContentEncoding::from("deflate,deflate".to_string()),
            ContentEncoding::from("gzip,gzip".to_string()),
            ContentEncoding::from("identity,identity".to_string()),
        ] {
            let content = idp_proto::serialize_and_encode_to_content(
                &dummy_data,
                content_format.clone(),
                content_encoding.clone(),
            )
            .expect("pass");
            log::debug!("content: {:?}", content);
            match std::str::from_utf8(content.content_byte_v.as_slice()) {
                Ok(s) => {
                    log::debug!("content.content_byte_v as string: {}", s);
                }
                _ => {
                    // The bytes aren't valid UTF-8, so don't bother printing them.
                }
            }

            let dummy_typed_body_deserialized =
                idp_proto::decode_and_deserialize_from_content::<DummyData>(&content)
                    .expect("pass");
            assert_eq!(dummy_typed_body_deserialized, dummy_data);
        }
    }
}

#[test]
fn test_contentifiable_3() {
    {
        let s = "this is us-ascii\nthingy\tblah";
        let content_format = ContentFormat::charset_us_ascii();
        let content_encoding = ContentEncoding::none();
        let content =
            serialize_and_encode_to_content(&s, content_format.clone(), content_encoding.clone())
                .expect("pass");
        log::debug!("&str content: {:?}", content);
        log::debug!("    has {:?}", content.content_metadata.content_type());
        assert_eq!(content.content_metadata.content_length, s.len() as u64);
        assert_eq!(content.content_metadata.content_class, s.content_class());
        assert_eq!(content.content_metadata.content_format, content_format);
        assert_eq!(content.content_metadata.content_encoding, content_encoding);
        assert_eq!(content.content_byte_v, s.as_bytes());
    }
    {
        let s = "this is also us-ascii\nthingy\tblah".to_string();
        let content_format = ContentFormat::charset_us_ascii();
        let content_encoding = ContentEncoding::none();
        let content =
            serialize_and_encode_to_content(&s, content_format.clone(), content_encoding.clone())
                .expect("pass");
        log::debug!("String content: {:?}", content);
        log::debug!("    has {:?}", content.content_metadata.content_type());
        assert_eq!(content.content_metadata.content_length, s.len() as u64);
        assert_eq!(content.content_metadata.content_class, s.content_class());
        assert_eq!(content.content_metadata.content_format, content_format);
        assert_eq!(content.content_metadata.content_encoding, content_encoding);
        assert_eq!(content.content_byte_v, s.as_bytes());
    }
    {
        let s = "this is utf-8\n日本語\tblah";
        let content_format = ContentFormat::charset_utf_8();
        let content_encoding = ContentEncoding::none();
        let content =
            serialize_and_encode_to_content(&s, content_format.clone(), content_encoding.clone())
                .expect("pass");
        log::debug!("&str content: {:?}", content);
        log::debug!("    has {:?}", content.content_metadata.content_type());
        assert_eq!(content.content_metadata.content_length, s.len() as u64);
        assert_eq!(content.content_metadata.content_class, s.content_class());
        assert_eq!(content.content_metadata.content_format, content_format);
        assert_eq!(content.content_metadata.content_encoding, content_encoding);
        assert_eq!(content.content_byte_v, s.as_bytes());
    }
    {
        let s = "this is also utf-8\n日本語\tblah".to_string();
        let content_format = ContentFormat::charset_utf_8();
        let content_encoding = ContentEncoding::none();
        let content =
            serialize_and_encode_to_content(&s, content_format.clone(), content_encoding.clone())
                .expect("pass");
        log::debug!("String content: {:?}", content);
        log::debug!("    has {:?}", content.content_metadata.content_type());
        assert_eq!(content.content_metadata.content_length, s.len() as u64);
        assert_eq!(content.content_metadata.content_class, s.content_class());
        assert_eq!(content.content_metadata.content_format, content_format);
        assert_eq!(content.content_metadata.content_encoding, content_encoding);
        assert_eq!(content.content_byte_v, s.as_bytes());
    }
}

#[test]
fn test_plum_relations_builder() {
    // Just a made-up PlumHeadSeal.
    let dependency_plum_head_seal = PlumHeadSeal {
        value: idp_proto::Seal {
            sha256sum: idp_proto::Sha256Sum {
                value: vec![23, 133, 144, 211],
            },
        },
    };
    let dummy_data = DummyData {
        name: "test_plum_relations_builder_name".to_string(),
        dependency: dependency_plum_head_seal.clone(),
    };

    // Make up a PlumBodySeal (this would not validate in a real Plum).
    let plum_body_seal = PlumBodySeal {
        value: idp_proto::Seal {
            sha256sum: idp_proto::Sha256Sum {
                value: vec![13, 5, 8, 4],
            },
        },
    };

    let plum_relations = PlumRelationsBuilder::new()
        .with_source_plum_body_seal(plum_body_seal.clone())
        .with_plum_relations_from(&dummy_data)
        .build()
        .expect("pass");
    log::debug!("plum_relations: {:?}", plum_relations);

    assert_eq!(plum_relations.plum_relation_flags_mapping_v.len(), 1);
    let plum_relation_flags_mapping = plum_relations
        .plum_relation_flags_mapping_v
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(plum_relations.source_plum_body_seal, plum_body_seal);
    assert_eq!(
        plum_relation_flags_mapping.target_plum_head_seal,
        dependency_plum_head_seal
    );
}

fn random_made_up_plum_head_seal() -> PlumHeadSeal {
    PlumHeadSeal {
        value: idp_proto::Seal {
            sha256sum: idp_proto::Sha256Sum {
                value: Uuid::new_v4().as_bytes().to_vec(),
            },
        },
    }
}

fn random_nonce() -> Nonce {
    Nonce {
        value: Uuid::new_v4().as_bytes().to_vec(),
    }
}

#[derive(Debug)]
struct TestPlumBuilderOptions {
    plum_head_nonce_o: Option<Nonce>,
    plum_metadata_nonce_o: Option<Nonce>,
    plum_created_at_o: Option<UnixNanoseconds>,
    plum_metadata_should_not_include_plum_body_content_metadata: bool,
    plum_metadata_additional_content_o: Option<Content>,
    plum_relations_nonce_o: Option<Nonce>,
    plum_body_nonce_o: Option<Nonce>,
}

fn create_test_content_and_plum(
    test_plum_builder_options: TestPlumBuilderOptions,
) -> (DummyData, Plum) {
    // Just a made-up PlumHeadSeal.
    let dependency_plum_head_seal = PlumHeadSeal {
        value: idp_proto::Seal {
            sha256sum: idp_proto::Sha256Sum {
                value: vec![2, 33, 44, 111],
            },
        },
    };
    let dummy_data = DummyData {
        name: "test_plum_builder_name".to_string(),
        dependency: dependency_plum_head_seal.clone(),
    };

    let mut plum_builder = PlumBuilder::new();
    if let Some(plum_head_nonce) = test_plum_builder_options.plum_head_nonce_o {
        plum_builder = plum_builder.with_plum_head_nonce(plum_head_nonce);
    }
    if let Some(plum_metadata_nonce) = test_plum_builder_options.plum_metadata_nonce_o {
        plum_builder = plum_builder.with_plum_metadata_nonce(plum_metadata_nonce);
    }
    if let Some(plum_created_at) = test_plum_builder_options.plum_created_at_o {
        plum_builder = plum_builder.with_plum_created_at(plum_created_at);
    }
    if test_plum_builder_options.plum_metadata_should_not_include_plum_body_content_metadata {
        plum_builder = plum_builder.plum_metadata_should_not_include_plum_body_content_metadata();
    }
    if let Some(plum_metadata_additional_content) =
        test_plum_builder_options.plum_metadata_additional_content_o
    {
        plum_builder =
            plum_builder.with_plum_metadata_additional_content(plum_metadata_additional_content);
    }
    if let Some(plum_relations_nonce) = test_plum_builder_options.plum_relations_nonce_o {
        plum_builder = plum_builder.with_plum_relations_nonce(plum_relations_nonce);
    }
    if let Some(plum_body_nonce) = test_plum_builder_options.plum_body_nonce_o {
        plum_builder = plum_builder.with_plum_body_nonce(plum_body_nonce);
    }
    let plum = plum_builder
        .with_plum_relations_and_plum_body_content_from(
            &dummy_data,
            ContentFormat::json(),
            ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");

    (dummy_data, plum)
}

#[test]
fn test_plum_builder() {
    PlumBuilder::new().build().expect_err("pass");

    // Use the same nonce so we get deterministic output, but show that using nonces in the different
    // places gives distinict PlumHeadSeals.  Note that you would never re-use a nonce in a real setting.
    // Use a short one so it produces legible output in the log.
    let nonce = Nonce::from(b"xy".to_vec());
    let nonce_ov = [None, Some(nonce)];
    let additional_content_string = "blahhhhhh".to_string();
    let additional_content = Content {
        content_metadata: ContentMetadata {
            content_length: additional_content_string.len() as u64,
            content_class: ContentClass::text_plain(),
            content_format: ContentFormat::charset_utf_8(),
            content_encoding: ContentEncoding::none(),
        },
        content_byte_v: additional_content_string.as_bytes().to_vec(),
    };

    let mut plum_head_seal_s = HashSet::new();
    let mut test_case_count = 0usize;

    // Test all combinations of options.
    for plum_head_nonce_o in nonce_ov.iter() {
        for plum_metadata_nonce_o in nonce_ov.iter() {
            for plum_created_at_o in [None, Some(UnixNanoseconds::from(123456789))].iter() {
                for plum_metadata_should_not_include_plum_body_content_metadata in [false, true] {
                    for plum_metadata_additional_content_o in
                        [None, Some(additional_content.clone())].iter()
                    {
                        for plum_relations_nonce_o in nonce_ov.iter() {
                            for plum_body_nonce_o in nonce_ov.iter() {
                                log::debug!("test case {} ---------", test_case_count);
                                let test_plum_builder_options = TestPlumBuilderOptions {
                                    plum_head_nonce_o: plum_head_nonce_o.clone(),
                                    plum_metadata_nonce_o: plum_metadata_nonce_o.clone(),
                                    plum_created_at_o: plum_created_at_o.clone(),
                                    plum_metadata_should_not_include_plum_body_content_metadata,
                                    plum_metadata_additional_content_o:
                                        plum_metadata_additional_content_o.clone(),
                                    plum_relations_nonce_o: plum_relations_nonce_o.clone(),
                                    plum_body_nonce_o: plum_body_nonce_o.clone(),
                                };
                                log::debug!(
                                    "test_plum_builder_options: {:?}",
                                    test_plum_builder_options
                                );
                                let (dummy_data, plum) =
                                    create_test_content_and_plum(test_plum_builder_options);

                                log::debug!("plum: {:?}", plum);
                                plum.verify().expect("pass");
                                assert_eq!(
                                    plum.plum_relations.plum_relation_flags_mapping_v.len(),
                                    1
                                );
                                let plum_relation_flags_mapping = plum
                                    .plum_relations
                                    .plum_relation_flags_mapping_v
                                    .into_iter()
                                    .next()
                                    .unwrap();
                                assert_eq!(
                                    plum_relation_flags_mapping.target_plum_head_seal,
                                    dummy_data.dependency
                                );

                                plum_head_seal_s.insert(PlumHeadSeal::from(&plum.plum_head));
                                test_case_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Because of the nonces, there should be one distinct PlumHeadSeal per test case.
    log::debug!("test_case_count: {}", test_case_count);
    assert_eq!(plum_head_seal_s.len(), test_case_count);
}
