#![allow(unused_imports)] // TEMP HACK

use anyhow::Result;
use idp_proto::{
    ContentType, ContentTypeable, Nonce, Plum, PlumBodyBuilder, PlumBodySeal, PlumBuilder,
    PlumHeadBuilder, PlumHeadSeal, PlumRelationFlags, PlumRelational,
};
use std::collections::HashMap;
use uuid::Uuid;

// Dummy type for use as a typed body.
#[derive(serde::Serialize)]
pub struct DummyTypedBody {
    pub name: String,
    pub content: PlumHeadSeal,
}

impl ContentTypeable for DummyTypedBody {
    fn content_type() -> ContentType {
        ContentType::from("idp_proto::tests::DummyTypedBody".as_bytes().to_vec())
    }
}

impl PlumRelational for DummyTypedBody {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        plum_relation_flags_m.insert(self.content.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
    }
}

#[test]
fn test_plum_builder() {
    let _ = env_logger::try_init();

    let plum = PlumBuilder::new()
        .with_plum_body_content_type(ContentType::from("text/plain".as_bytes().to_vec()))
        .with_plum_body_content(
            format!("test_plum_builder, {}.", Uuid::new_v4())
                .as_bytes()
                .to_vec(),
        )
        .build()
        .expect("pass");
    let plum_head_seal = PlumHeadSeal::from(&plum);

    log::debug!("plum: {:?}", plum);
    log::debug!("plum_head_seal: {:?}", plum_head_seal);

    let data_2 = DummyTypedBody {
        name: "thingy2".into(),
        content: plum_head_seal.clone(),
    };
    let plum_2 = PlumBuilder::new()
        .with_relational_typed_content_from(&data_2)
        .expect("pass")
        .build()
        .expect("pass");
    let plum_2_head_seal = PlumHeadSeal::from(&plum_2);
    log::debug!("plum_2: {:?}", plum_2);
    log::debug!("plum_2_head_seal: {:?}", plum_2_head_seal);
    // Verify that use of the various nonces causes the PlumHeadSeal value to change.
    {
        let plum_2r = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_relations_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2r_head_seal = PlumHeadSeal::from(&plum_2r);

        let plum_2b = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_body_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2b_head_seal = PlumHeadSeal::from(&plum_2b);

        let plum_2h = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_head_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2h_head_seal = PlumHeadSeal::from(&plum_2h);

        let plum_2br = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_body_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .with_plum_relations_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2br_head_seal = PlumHeadSeal::from(&plum_2br);

        let plum_2hr = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_head_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .with_plum_relations_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2hr_head_seal = PlumHeadSeal::from(&plum_2hr);

        let plum_2bh = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_body_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .with_plum_head_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2bh_head_seal = PlumHeadSeal::from(&plum_2bh);

        let plum_2bhr = PlumBuilder::new()
            .with_relational_typed_content_from(&data_2)
            .expect("pass")
            .with_plum_body_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .with_plum_head_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .with_plum_relations_nonce(Nonce::from(vec![1u8, 2u8, 3u8, 4u8]))
            .build()
            .expect("pass");
        let plum_2bhr_head_seal = PlumHeadSeal::from(&plum_2bhr);

        assert_ne!(plum_2_head_seal, plum_2b_head_seal);
        assert_ne!(plum_2_head_seal, plum_2h_head_seal);
        assert_ne!(plum_2_head_seal, plum_2r_head_seal);
        assert_ne!(plum_2_head_seal, plum_2bh_head_seal);
        assert_ne!(plum_2_head_seal, plum_2br_head_seal);
        assert_ne!(plum_2_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2b_head_seal, plum_2h_head_seal);
        assert_ne!(plum_2b_head_seal, plum_2r_head_seal);
        assert_ne!(plum_2b_head_seal, plum_2bh_head_seal);
        assert_ne!(plum_2b_head_seal, plum_2br_head_seal);
        assert_ne!(plum_2b_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2b_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2h_head_seal, plum_2r_head_seal);
        assert_ne!(plum_2h_head_seal, plum_2bh_head_seal);
        assert_ne!(plum_2h_head_seal, plum_2br_head_seal);
        assert_ne!(plum_2h_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2h_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2r_head_seal, plum_2bh_head_seal);
        assert_ne!(plum_2r_head_seal, plum_2br_head_seal);
        assert_ne!(plum_2r_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2r_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2bh_head_seal, plum_2br_head_seal);
        assert_ne!(plum_2bh_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2bh_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2br_head_seal, plum_2hr_head_seal);
        assert_ne!(plum_2br_head_seal, plum_2bhr_head_seal);

        assert_ne!(plum_2hr_head_seal, plum_2bhr_head_seal);
    }

    let data_3 = DummyTypedBody {
        name: "thingy3".into(),
        content: plum_head_seal.clone(),
    };
    let plum_3 = PlumBuilder::new()
        .with_relational_untyped_content_from(&data_3)
        .expect("pass")
        .build()
        .expect("pass");
    let plum_3_head_seal = PlumHeadSeal::from(&plum_3);
    log::debug!("plum_3: {:?}", plum_3);
    log::debug!("plum_3_head_seal: {:?}", plum_3_head_seal);

    let data_4 = DummyTypedBody {
        name: "thingy4".into(),
        content: plum_head_seal.clone(),
    };
    let plum_4 = PlumBuilder::new()
        .with_nonrelational_typed_content_from(&data_4)
        .expect("pass")
        .build()
        .expect("pass");
    let plum_4_head_seal = PlumHeadSeal::from(&plum_4);
    log::debug!("plum_4: {:?}", plum_4);
    log::debug!("plum_4_head_seal: {:?}", plum_4_head_seal);

    let data_5 = DummyTypedBody {
        name: "thingy5".into(),
        content: plum_head_seal.clone(),
    };
    let plum_5 = PlumBuilder::new()
        .with_nonrelational_untyped_content_from(&data_5)
        .expect("pass")
        .build()
        .expect("pass");
    let plum_5_head_seal = PlumHeadSeal::from(&plum_5);
    log::debug!("plum_5: {:?}", plum_5);
    log::debug!("plum_5_head_seal: {:?}", plum_5_head_seal);

    assert_ne!(plum_head_seal, plum_2_head_seal);
    assert_ne!(plum_head_seal, plum_3_head_seal);
    assert_ne!(plum_head_seal, plum_4_head_seal);
    assert_ne!(plum_head_seal, plum_5_head_seal);

    assert_ne!(plum_2_head_seal, plum_3_head_seal);
    assert_ne!(plum_2_head_seal, plum_4_head_seal);
    assert_ne!(plum_2_head_seal, plum_5_head_seal);

    assert_ne!(plum_3_head_seal, plum_4_head_seal);
    assert_ne!(plum_3_head_seal, plum_5_head_seal);

    assert_ne!(plum_4_head_seal, plum_5_head_seal);
}
