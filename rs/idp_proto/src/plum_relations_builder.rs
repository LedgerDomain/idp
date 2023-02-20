use crate::{
    Nonce, PlumBodySeal, PlumHeadSeal, PlumRelationFlags, PlumRelationFlagsMapping, PlumRelational,
    PlumRelations,
};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct PlumRelationsBuilder {
    has_relations: bool,
    plum_relations_nonce_o: Option<Nonce>,
    // This is the PlumBodySeal of the PlumBody that these relations originate from.
    source_plum_body_seal_o: Option<PlumBodySeal>,
    // A BTreeMap is used here so that the order of elements is deterministic.
    plum_relation_flags_m: BTreeMap<PlumHeadSeal, PlumRelationFlags>,
}

impl PlumRelationsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Builds a PlumRelations.
    pub fn build(self) -> Result<Option<PlumRelations>> {
        if self.has_relations {
            let source_plum_body_seal =
                self.source_plum_body_seal_o.ok_or_else(
                    || anyhow::anyhow!("PlumRelationsBuilder::build can't proceed unless with_plum_relations_from was used to specify the PlumBody that the relations originate from")
                )?;
            // Consume the BTreeMap in deterministic order and produce Vec.
            Ok(Some(PlumRelations {
                plum_relations_nonce_o: self.plum_relations_nonce_o,
                source_plum_body_seal,
                plum_relation_flags_mapping_v: self
                    .plum_relation_flags_m
                    .into_iter()
                    .map(
                        |(target_plum_head_seal, plum_relation_flags)| PlumRelationFlagsMapping {
                            target_plum_head_seal,
                            plum_relation_flags_raw: plum_relation_flags.into(),
                        },
                    )
                    .collect(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Specifies the plum_relations_nonce_o field for use in resisting dictionary attacks.  Default is no nonce.
    pub fn with_plum_relations_nonce(mut self, plum_relations_nonce: Nonce) -> Self {
        self.has_relations = true;
        self.plum_relations_nonce_o = Some(plum_relations_nonce);
        self
    }
    pub fn with_source_plum_body_seal(mut self, source_plum_body_seal: PlumBodySeal) -> Self {
        self.has_relations = true;
        self.source_plum_body_seal_o = Some(source_plum_body_seal);
        self
    }
    /// Derives the plum_relation_flags_mapping_v field using a content whose type implements Relational.
    /// Default is no plum_relations.
    pub fn with_plum_relations_from<B>(mut self, content: &B) -> Self
    where
        B: PlumRelational,
    {
        self.has_relations = true;

        let mut plum_relation_flags_m: HashMap<PlumHeadSeal, PlumRelationFlags> = HashMap::new();
        content.accumulate_plum_relations_nonrecursive(&mut plum_relation_flags_m);
        for (target_plum_head_seal, plum_relation_flags) in plum_relation_flags_m.into_iter() {
            self.plum_relation_flags_m
                .insert(target_plum_head_seal, plum_relation_flags);
        }
        self
    }
}
