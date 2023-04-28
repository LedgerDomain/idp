use crate::{Hashable, PlumRelations};

impl Hashable for PlumRelations {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.plum_relations_nonce_o.update_hasher(hasher);
        self.source_plum_body_seal.update_hasher(hasher);
        // Note that len() produces usize, but that's hashed as a u64 (which is represented in little-endian order).
        self.plum_relation_flags_mapping_v
            .len()
            .update_hasher(hasher);
        // Iterate over the vector, hashing each element.
        for plum_relation_flags_mapping in self.plum_relation_flags_mapping_v.iter() {
            plum_relation_flags_mapping.update_hasher(hasher);
        }
    }
}
