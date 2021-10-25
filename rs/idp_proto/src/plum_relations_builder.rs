use crate::{Nonce, PlumHeadSeal, PlumRelations, PlumRelationFlagsMapping, Relational, RelationFlags};
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct PlumRelationsBuilder {
    has_relations: bool,
    relations_nonce_o: Option<Nonce>,
    // A BTreeMap is used here so that the order of elements is deterministic.
    relation_flags_m: BTreeMap<PlumHeadSeal, RelationFlags>,
}

impl PlumRelationsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Builds a PlumRelations.
    pub fn build(self) -> Option<PlumRelations> {
        if self.has_relations {
            // Consume the BTreeMap in deterministic order and produce Vec.
            Some(PlumRelations {
                relations_nonce_o: self.relations_nonce_o,
                relation_flags_mappings:
                    self.relation_flags_m
                        .into_iter()
                        .map(
                            |(target_head_seal, relation_flags)| {
                                PlumRelationFlagsMapping {
                                    target_head_seal,
                                    relation_flags_raw: relation_flags.into(),
                                }
                            }
                        )
                        .collect(),
            })
        } else {
            None
        }
    }

    /// Specifies the relations_nonce_o field for use in resisting dictionary attacks.  Default is no nonce.
    pub fn with_relations_nonce(mut self, relations_nonce: Nonce) -> Self {
        self.has_relations = true;
        self.relations_nonce_o = Some(relations_nonce);
        self
    }
    /// Derives the relation_flags_mappings field using a content whose type implements Relational.
    /// Default is no relations.
    pub fn with_relations_from<B>(mut self, content: &B) -> Self
    where B: Relational {
        self.has_relations = true;

        let mut relation_flags_m: HashMap<PlumHeadSeal, RelationFlags> = HashMap::new();
        content.accumulate_relations_nonrecursive(&mut relation_flags_m);
        for (target_head_seal, relation_flags) in relation_flags_m.into_iter() {
            self.relation_flags_m.insert(target_head_seal, relation_flags);
        }
        self
    }
}

