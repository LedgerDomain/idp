pub use anyhow::Result;

/// Get the did:key:<method-specific-identifier> from a (public) JWK.
pub fn did_key_from_jwk(jwk: &ssi_jwk::JWK) -> Result<ssi_dids::PrimaryDIDURL> {
    use ssi_dids::DIDMethod;
    use std::str::FromStr;
    Ok(ssi_dids::PrimaryDIDURL::from_str(
        &did_method_key::DIDKey
            .generate(&ssi_dids::Source::Key(jwk))
            .ok_or_else(|| anyhow::anyhow!("unable to derive did:key from JWK"))?,
    )?)
}

/// Produce a DIDURL with the given primary DIDURL and the multibase portion of the given did:key as its fragment.
pub fn with_multibase_fragment(
    primary_did: ssi_dids::PrimaryDIDURL,
    did_key_for_multibase_fragment: &str,
) -> ssi_dids::DIDURL {
    // TODO: Validate did_key_for_multibase_fragment is a valid did:key DIDURL (otherwise the unwraps might fail)
    ssi_dids::DIDURL::try_from(
        primary_did.with_fragment(
            did_key_for_multibase_fragment
                .split(':')
                .nth(2)
                .unwrap()
                .to_string(),
        ),
    )
    .unwrap()
}

lazy_static::lazy_static! {
    /// The set of supported DID methods.  Use "did-ethr" and "did-web" features of this crate
    /// to enable those methods in DID resolution.
    static ref DID_METHODS: ssi_dids::DIDMethods<'static> = {
        let mut methods = ssi_dids::DIDMethods::default();
        // methods.insert(&::did_ethr::DIDEthr);
        methods.insert(Box::new(::did_method_key::DIDKey));
        // methods.insert(&::did_web::DIDWeb);
        methods
    };
    /// The DID resolver for the set of supported DID methods.
    static ref DID_RESOLVER: ssi_dids::did_resolve::SeriesResolver<'static> = {
        ssi_dids::did_resolve::SeriesResolver {
            resolvers: vec![DID_METHODS.to_resolver()],
        }
    };
}

/// Return the singleton DID resolver.  This only supports did:key.
pub fn did_resolver() -> &'static dyn ssi_dids::did_resolve::DIDResolver {
    let did_resolver: &ssi_dids::did_resolve::SeriesResolver = &DID_RESOLVER;
    did_resolver
}
