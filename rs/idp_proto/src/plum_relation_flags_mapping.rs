use crate::{Hashable, PlumRelationFlagsMapping};

impl Hashable for PlumRelationFlagsMapping {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.target_plum_head_seal.update_hasher(hasher);
        self.plum_relation_flags_raw.update_hasher(hasher);
    }
}
