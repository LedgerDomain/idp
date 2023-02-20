use idp_proto::{
    Id, Nonce, PlumBody, PlumBodySeal, PlumHead, PlumRelations, PlumRelationsSeal, Seal, Sha256Sum,
    UnixNanoseconds,
};

pub struct PlumHeadsRow {
    pub plum_heads_rowid: i64,
    pub row_inserted_at: i64,
    pub plum_head_seal: Vec<u8>,
    pub plum_head_nonce_o: Option<Vec<u8>>,
    pub plum_relations_seal_o: Option<Vec<u8>>,
    pub plum_body_seal: Vec<u8>,
    pub owner_id_o: Option<String>,
    pub created_at_o: Option<i64>,
    pub metadata_o: Option<Vec<u8>>,
}

pub struct MinimalPlumHeadsRow {
    pub plum_head_nonce_o: Option<Vec<u8>>,
    pub plum_relations_seal_o: Option<Vec<u8>>,
    pub plum_body_seal: Vec<u8>,
    pub owner_id_o: Option<String>,
    pub created_at_o: Option<i64>,
    pub metadata_o: Option<Vec<u8>>,
}

// TODO: This should really be TryFrom, because it's possible the data in the DB can get corrupted.
impl From<MinimalPlumHeadsRow> for PlumHead {
    fn from(minimal_plum_heads_row: MinimalPlumHeadsRow) -> Self {
        Self {
            plum_head_nonce_o: minimal_plum_heads_row.plum_head_nonce_o.map(Nonce::from),
            plum_relations_seal_o: minimal_plum_heads_row
                .plum_relations_seal_o
                .map(Sha256Sum::from)
                .map(Seal::from)
                .map(PlumRelationsSeal::from),
            plum_body_seal: PlumBodySeal::from(Seal::from(Sha256Sum::from(
                minimal_plum_heads_row.plum_body_seal,
            ))),
            owner_id_o: minimal_plum_heads_row.owner_id_o.map(Id::from),
            created_at_o: minimal_plum_heads_row
                .created_at_o
                .map(UnixNanoseconds::from),
            metadata_o: minimal_plum_heads_row.metadata_o,
        }
    }
}

pub struct PlumRelationsRow {
    pub plum_relations_rowid: i64,
    pub row_inserted_at: i64,
    pub plum_relations_seal: Vec<u8>,
    pub plum_relations_nonce_o: Option<Vec<u8>>,
    pub source_plum_body_seal: Vec<u8>,
}

pub struct PlumRelationMappingsRow {
    pub plum_relation_mappings_rowid: i64,
    pub plum_relations_rowid: i64,
    pub target_plum_head_seal: Vec<u8>,
    pub plum_relation_flags: i32,
}

pub struct PlumBodiesRow {
    pub plum_bodies_rowid: i64,
    pub row_inserted_at: i64,
    pub plum_body_seal: Vec<u8>,
    pub plum_body_nonce_o: Option<Vec<u8>>,
    pub plum_body_content_length: i64,
    pub plum_body_content_type: Vec<u8>,
    pub plum_body_content: Vec<u8>,
}
