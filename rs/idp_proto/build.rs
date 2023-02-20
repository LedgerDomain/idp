fn main() -> Result<(), Box<dyn std::error::Error>> {
    // trigger recompilation only when the proto dir is changed.
    println!("cargo:rerun-if-changed=proto");

    // This allows customization of the generated Rust code, in particular trait derivation.
    tonic_build::configure()
        .out_dir("src/generated")
        .type_attribute("idp.ContentType", "#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]")
        .type_attribute("idp.Id", "#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]")
        .type_attribute("idp.Nonce", "#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]")
        .type_attribute("idp.PlumBodySeal", "#[derive(derive_more::Deref, serde::Deserialize, derive_more::From, serde::Serialize)]")
        .type_attribute("idp.PlumHeadSeal", "#[derive(derive_more::Deref, serde::Deserialize, Eq, derive_more::From, Hash, Ord, PartialOrd, serde::Serialize)]")
        .type_attribute("idp.PlumRelation", "#[derive(serde::Deserialize, num_derive::FromPrimitive, serde::Serialize)]")
        .type_attribute("idp.PlumRelationFlagsRaw", "#[derive(Copy, serde::Deserialize, serde::Serialize)]")
        .type_attribute("idp.PlumRelationsSeal", "#[derive(derive_more::Deref, serde::Deserialize, Eq, derive_more::From, Hash, Ord, PartialOrd, serde::Serialize)]")
        .type_attribute("idp.Seal", "#[derive(derive_more::Deref, serde::Deserialize, Eq, derive_more::From, Hash, Ord, PartialOrd, serde::Serialize)]")
        .type_attribute("idp.Sha256Sum", "#[derive(derive_more::Deref, serde::Deserialize, Eq, derive_more::From, Hash, Ord, PartialOrd, serde::Serialize)]")
        .type_attribute("idp.UnixNanoseconds", "#[derive(Copy, serde::Deserialize, derive_more::From, derive_more::Into, serde::Serialize)]")
        .build_client(true)
        .client_mod_attribute("idp", "#[cfg(feature = \"client\")]")
        .build_server(true)
        .server_mod_attribute("idp", "#[cfg(feature = \"server\")]")
        .compile(&["proto/idp.proto"], &["proto"])?;

    Ok(())
}
