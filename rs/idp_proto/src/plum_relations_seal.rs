use crate::{PlumRelationFlagsMapping, PlumRelations, PlumRelationsSeal, Seal, Sha256Sum};

impl AsRef<[u8]> for PlumRelationsSeal {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

impl std::fmt::Display for PlumRelationsSeal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.value)
    }
}

impl From<&PlumRelations> for PlumRelationsSeal {
    fn from(plum_relations: &PlumRelations) -> PlumRelationsSeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        // NOTE: The specific order and form of this hashing must NOT be changed!

        if let Some(plum_relations_nonce) = &plum_relations.plum_relations_nonce_o {
            hasher.update(b"\x01");
            hasher.update(&plum_relations_nonce.value);
        } else {
            hasher.update(b"\x00");
        }

        hasher.update(&plum_relations.source_plum_body_seal.value.sha256sum.value);

        // to_le_bytes gives little-endian representation.
        hasher.update((plum_relations.plum_relation_flags_mapping_v.len() as u64).to_le_bytes());
        for PlumRelationFlagsMapping {
            target_plum_head_seal,
            plum_relation_flags_raw,
        } in &plum_relations.plum_relation_flags_mapping_v
        {
            hasher.update(&target_plum_head_seal.value.sha256sum.value);
            // Note that plum_relation_flags_raw.value is u32.
            hasher.update(plum_relation_flags_raw.value.to_le_bytes());
        }

        PlumRelationsSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}
