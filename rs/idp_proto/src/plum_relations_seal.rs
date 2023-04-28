use crate::{Hashable, PlumRelations, PlumRelationsSeal, Seal, Sha256Sum};

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

        plum_relations.update_hasher(&mut hasher);

        PlumRelationsSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}

impl Hashable for PlumRelationsSeal {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
