use crate::schema::{plum_bodies, plum_heads, plum_relations};
use idp_proto::{
    ContentType, Did, Nonce, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumRelationsSeal,
    RelationFlags, UnixSeconds,
};

//
// plum_heads table
//

#[derive(Clone, Debug, PartialEq, diesel::Queryable)]
pub struct PlumHeadRow {
    /// Primary key
    pub head_seal: PlumHeadSeal,

    /// This is the timestamp at which this row was inserted.  It is not the owner-made timestamp.
    pub row_inserted_at: UnixSeconds,

    /// This is the primary key for the PlumBody this PlumHead refers to.
    pub body_seal: PlumBodySeal,
    /// This is the number of bytes in the PlumBody's content.
    pub body_length: i64,
    /// This is the ContentType for the PlumBody, needed to understand its content.
    pub body_content_type: ContentType,
    /// This is an optional Nonce that can be used to mitigate against dictionary attacks.
    pub head_nonce_o: Option<Nonce>,
    /// This optionally identifies the owner of this Plum.
    pub owner_did_o: Option<Did>,
    /// This is the optional timestamp at which this Plum was created.
    pub created_at_o: Option<UnixSeconds>,
    /// This is the optional, unstructured metadata for this Plum.
    pub metadata_o: Option<Vec<u8>>,
    /// This is the optional seal for the PlumRelations for this Plum.  If this is None, then
    /// this Plum is considered to be opaque with respect to relations, and they'll have to
    /// be derived separately.
    pub relations_seal_o: Option<PlumRelationsSeal>,
}

impl std::convert::Into<PlumHead> for PlumHeadRow {
    fn into(self) -> PlumHead {
        if self.body_length < 0 {
            panic!("PlumHeadRow body_length field was negative.");
        }
        PlumHead {
            body_seal: self.body_seal,
            body_length: self.body_length as u64,
            body_content_type: self.body_content_type,
            head_nonce_o: self.head_nonce_o,
            owner_did_o: self.owner_did_o,
            created_at_o: self.created_at_o,
            metadata_o: self.metadata_o,
            relations_seal_o: self.relations_seal_o,
        }
    }
}

/// Note that row_inserted_at will take DEFAULT CURRENT_TIMESTAMP
#[derive(diesel::Insertable)]
#[table_name = "plum_heads"]
pub struct PlumHeadRowInsertion<'a> {
    pub head_seal: PlumHeadSeal,
    pub body_seal: &'a PlumBodySeal,
    pub body_length: i64,
    pub body_content_type: &'a ContentType,
    pub head_nonce_o: Option<&'a Nonce>,
    pub owner_did_o: Option<&'a Did>,
    pub created_at_o: Option<&'a UnixSeconds>,
    // NOTE: This should be Option<&'a [u8]>, but I was having trouble getting that to work in
    // the implementation of std::convert::From<&'a PlumHead> for PlumHeadRowInsertion<'a>
    pub metadata_o: Option<&'a Vec<u8>>,
    pub relations_seal_o: Option<&'a PlumRelationsSeal>,
}

impl<'a> std::convert::From<&'a PlumHead> for PlumHeadRowInsertion<'a> {
    fn from(plum_head: &'a PlumHead) -> Self {
        if plum_head.body_length > i64::MAX as u64 {
            // TODO: Maybe make a u63 type which is defined as the overlap of i64 and u64
            panic!("plum_head.body_length (which was {}) exceeded maximum acceptable value (which is {})", plum_head.body_length, i64::MAX);
        }
        PlumHeadRowInsertion {
            head_seal: PlumHeadSeal::from(plum_head),
            body_seal: &plum_head.body_seal,
            body_length: plum_head.body_length as i64,
            body_content_type: &plum_head.body_content_type,
            head_nonce_o: plum_head.head_nonce_o.as_ref(),
            owner_did_o: plum_head.owner_did_o.as_ref(),
            created_at_o: plum_head.created_at_o.as_ref(),
            // metadata_o: if let Some(metadata) = plum_head.metadata_o { Some(metadata.as_slice()) } else { None },
            metadata_o: plum_head.metadata_o.as_ref(),
            relations_seal_o: plum_head.relations_seal_o.as_ref(),
        }
    }
}

//
// plum_relations table
//

/// Note that this row doesn't correspond to a single PlumRelations.  A PlumRelations instance
/// is unpacked by Datahost and creates one PlumRelationRow per (source, target) pair, where
/// there can be potentially many distinct relations between that source and target.  The subset
/// of relations is representing using a bitflag field.  A single relation has the form
/// `source --relation--> target` (analogous to SVO).  In this case, a whole subset of relation
/// values is stored in the row, meaning that multiple `source --relation--> target` relations
/// are represented by a single row.
#[derive(Clone, Debug, PartialEq, diesel::Queryable)]
pub struct PlumRelationsRow {
    /// This is the timestamp at which this row was inserted.  It is not the owner-made timestamp.
    pub row_inserted_at: UnixSeconds,

    /// This is the PlumHeadSeal of the "source" Plum.  This is the subject of the relation, i.e.
    /// in `source --relation--> target`, it's `source`.
    pub source_head_seal: PlumHeadSeal,
    /// This is the PlumHeadSeal of the "target" Plum.  This is the object of the relation, i.e.
    /// in `source --relation--> target`, it's `target`.
    pub target_head_seal: PlumHeadSeal,
    /// This is the subset of relations itself, i.e. for each bit set in relation_flags, a single
    /// relation is denoted, which corresponds to `relation` in `source --relation--> target`.
    pub relation_flags: RelationFlags,
}

/// Note that row_inserted_at will take DEFAULT CURRENT_TIMESTAMP.
#[derive(Debug, diesel::Insertable)]
#[table_name = "plum_relations"]
pub struct PlumRelationsRowInsertion {
    pub source_head_seal: PlumHeadSeal,
    pub target_head_seal: PlumHeadSeal,
    pub relation_flags: RelationFlags,
}

//
// plum_bodies table
//

#[derive(Clone, Debug, PartialEq, diesel::Queryable)]
pub struct PlumBodyRow {
    /// Primary key
    pub body_seal: PlumBodySeal,

    /// This is the timestamp at which this row was inserted.
    pub row_inserted_at: UnixSeconds,

    /// This is an optional Nonce that can be used to mitigate against dictionary attacks.
    pub body_nonce_o: Option<Nonce>,
    /// This is the actual content of the Plum, which will be None if and only if this
    /// row has been deleted.
    pub body_content_o: Option<Vec<u8>>,
}

impl std::convert::TryInto<PlumBody> for PlumBodyRow {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<PlumBody, Self::Error> {
        anyhow::ensure!(
            self.body_content_o.is_some(),
            "Can't turn PlumBodyRow into PlumBody when the body_content_o column value is NULL"
        );
        Ok(PlumBody {
            body_nonce_o: self.body_nonce_o,
            body_content: self.body_content_o.unwrap(),
        })
    }
}

/// Note that row_inserted_at will take DEFAULT CURRENT_TIMESTAMP.
#[derive(diesel::Insertable)]
#[table_name = "plum_bodies"]
pub struct PlumBodyRowInsertion<'a> {
    pub body_seal: PlumBodySeal,
    pub body_nonce_o: Option<&'a Nonce>,
    pub body_content_o: Option<&'a [u8]>,
}

impl<'a> std::convert::From<&'a PlumBody> for PlumBodyRowInsertion<'a> {
    fn from(plum_body: &'a PlumBody) -> Self {
        PlumBodyRowInsertion {
            body_seal: PlumBodySeal::from(plum_body),
            body_nonce_o: plum_body.body_nonce_o.as_ref(),
            body_content_o: Some(&plum_body.body_content),
        }
    }
}
