fn main() -> Result<(), Box<dyn std::error::Error>> {
    // trigger recompilation only when the proto dir is changed.
    // println!("cargo:rerun-if-changed=proto/idp.proto");
    println!("cargo:rerun-if-changed=proto");

    // This allows customization of the generated Rust code, in particular trait derivation.
    // TODO: Derive Debug traits for everything.
    tonic_build::configure()
        // // Not sure what this does, it doesn't seem to change stuff like
        // // `::prost::alloc::vec::Vec<u8>` to `Vec<u8>`.  But maybe that doesn't matter.
        // .compile_well_known_types(true)
        .out_dir("src/generated")
        // TODO: Add conditional build as features
        // .build_client(false)
        // .build_server(false)

        .type_attribute("idp.ContentType", "#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.ContentType", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.ContentType", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.ContentType", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.Did", "#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.Did", "#[diesel(deserialize_as = \"String\")]")
        .type_attribute("idp.Did", "#[diesel(serialize_as = \"String\")]")
        .type_attribute("idp.Did", "#[sql_type = \"diesel::sql_types::Text\"]")

        .type_attribute("idp.Nonce", "#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.Nonce", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Nonce", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Nonce", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.PlumBodySeal", "#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.PlumBodySeal", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumBodySeal", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumBodySeal", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.PlumHeadSeal", "#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.PlumHeadSeal", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumHeadSeal", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumHeadSeal", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.PlumRelationsSeal", "#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.PlumRelationsSeal", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumRelationsSeal", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.PlumRelationsSeal", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.Relation", "#[derive(diesel::AsExpression, num_derive::FromPrimitive, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.Relation", "#[diesel(deserialize_as = \"i32\")]")
        .type_attribute("idp.Relation", "#[diesel(serialize_as = \"i32\")]")
        .type_attribute("idp.Relation", "#[sql_type = \"diesel::sql_types::Integer\"]")

        .type_attribute("idp.RelationFlagsRaw", "#[derive(diesel::AsExpression, Copy, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.RelationFlagsRaw", "#[diesel(deserialize_as = \"i32\")]")
        .type_attribute("idp.RelationFlagsRaw", "#[diesel(serialize_as = \"i32\")]")
        .type_attribute("idp.RelationFlagsRaw", "#[sql_type = \"diesel::sql_types::Integer\"]")

        .type_attribute("idp.Seal", "#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.Seal", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Seal", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Seal", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.Sha256Sum", "#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.Sha256Sum", "#[diesel(deserialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Sha256Sum", "#[diesel(serialize_as = \"Vec<u8>\")]")
        .type_attribute("idp.Sha256Sum", "#[sql_type = \"diesel::sql_types::Binary\"]")

        .type_attribute("idp.UnixSeconds", "#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.UnixSeconds", "#[diesel(deserialize_as = \"i64\")]")
        .type_attribute("idp.UnixSeconds", "#[diesel(serialize_as = \"i64\")]")
        .type_attribute("idp.UnixSeconds", "#[sql_type = \"diesel::sql_types::BigInt\"]")

        .compile(
            &[
                "proto/idp.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
