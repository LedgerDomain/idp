use crate::{
    Nonce, PlumBodySeal, PlumHeadSeal, PlumRelationFlags, PlumRelationFlagsMapping, PlumRelational,
    PlumRelations,
};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct PlumRelationsBuilder {
    plum_relations_nonce_o: Option<Nonce>,
    // This is the PlumBodySeal of the PlumBody that these relations originate from.
    source_plum_body_seal_o: Option<PlumBodySeal>,
    // A BTreeMap is used here so that the order of elements is deterministic.
    plum_relation_flags_mo: Option<BTreeMap<PlumHeadSeal, PlumRelationFlags>>,
}

impl PlumRelationsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Builds a PlumRelations.
    pub fn build(self) -> Result<PlumRelations> {
        anyhow::ensure!(self.source_plum_body_seal_o.is_some(), "PlumRelationsBuilder::build can't proceed unless with_source_plum_body_seal was used to specify the PlumBody that the relations originate from");
        anyhow::ensure!(self.plum_relation_flags_mo.is_some(), "PlumRelationsBuilder::build can't proceed unless with_plum_relations_from was used to derive the plum relations");
        let source_plum_body_seal = self.source_plum_body_seal_o.unwrap();
        let plum_relation_flags_m = self.plum_relation_flags_mo.unwrap();
        // Consume the BTreeMap in deterministic order and produce Vec.
        Ok(PlumRelations {
            plum_relations_nonce_o: self.plum_relations_nonce_o,
            source_plum_body_seal,
            plum_relation_flags_mapping_v: plum_relation_flags_m
                .into_iter()
                .map(
                    |(target_plum_head_seal, plum_relation_flags)| PlumRelationFlagsMapping {
                        target_plum_head_seal,
                        plum_relation_flags_raw: plum_relation_flags.into(),
                    },
                )
                .collect(),
        })
    }

    /// Specifies the plum_relations_nonce_o field for use in resisting dictionary attacks.  Default is no nonce.
    pub fn with_plum_relations_nonce(mut self, plum_relations_nonce: Nonce) -> Self {
        self.plum_relations_nonce_o = Some(plum_relations_nonce);
        self
    }
    /// Specifies the source_plum_body_seal field directly.
    pub fn with_source_plum_body_seal(mut self, source_plum_body_seal: PlumBodySeal) -> Self {
        self.source_plum_body_seal_o = Some(source_plum_body_seal);
        self
    }
    /// Derives the plum_relation_flags_mapping_v field using a content whose type implements Relational.
    /// Default is no plum_relations.
    pub fn with_plum_relations_from<B>(mut self, content: &B) -> Self
    where
        B: PlumRelational,
    {
        // Collection happens using HashMap, but PlumRelationsBuilder uses BTreeMap to ensure deterministic order.
        let mut plum_relation_flags_m: HashMap<PlumHeadSeal, PlumRelationFlags> = HashMap::new();
        content.accumulate_plum_relations_nonrecursive(&mut plum_relation_flags_m);
        self.plum_relation_flags_mo = Some(plum_relation_flags_m.into_iter().collect());
        self
    }
}
