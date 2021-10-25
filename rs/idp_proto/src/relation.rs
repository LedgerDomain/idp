use crate::{PlumHeadSeal, Relation, RelationFlagsRaw};
use std::{collections::HashMap, convert::TryFrom};

// pub enum Relation {
//     ContentDependency   = 0,
//     MetadataDependency  = 1,
// }

// TODO: Ideally this type would be used directly by the code generated from idp.proto, instead
// of using this lame intermediate RelationFlagsRaw type and converting.
bitflags::bitflags! {
    #[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
    #[diesel(deserialize_as = "i32")]
    #[diesel(serialize_as = "i32")]
    #[sql_type = "diesel::sql_types::Integer"]
    pub struct RelationFlags: u32 {
        const CONTENT_DEPENDENCY    = 1u32 << (Relation::ContentDependency as u32);
        const METADATA_DEPENDENCY   = 1u32 << (Relation::MetadataDependency as u32);

        const NONE                  = 0;
        const ALL                   = Self::CONTENT_DEPENDENCY.bits | Self::METADATA_DEPENDENCY.bits;
    }
}

/// Convert a single Relation into its RelationFlags counterpart.  The reverse is not
/// possible in general.
impl std::convert::From<Relation> for RelationFlags {
    fn from(relation: Relation) -> Self {
        RelationFlags { bits: 1u32 << (relation as u32) }
    }
}

/// Convert from the lame RelationFlagsRaw type.  If it's possible to use RelationFlags directly
/// in the generated idp.proto code, then this wouldn't be necessary.
impl std::convert::TryFrom<RelationFlagsRaw> for RelationFlags {
    type Error = failure::Error;
    fn try_from(relation_flags_raw: RelationFlagsRaw) -> Result<Self, Self::Error> {
        if relation_flags_raw.value & !(RelationFlags::ALL.bits as u32) != 0 {
            return Err(failure::format_err!("RelationFlagsRaw value {:x} out of range (full bitmask is {:x})", relation_flags_raw.value, RelationFlags::ALL.bits as u32));
        }
        Ok(RelationFlags { bits: relation_flags_raw.value })
    }
}

/// Convert into the lame RelationFlagsRaw type.  If it's possible to use RelationFlags directly
/// in the generated idp.proto code, then this wouldn't be necessary.
impl std::convert::Into<RelationFlagsRaw> for RelationFlags {
    fn into(self) -> RelationFlagsRaw {
        RelationFlagsRaw { value: self.bits }
    }
}

/// This trait defines how to derive relations for a given type.
pub trait Relational {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_flags_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
    );
}

//
// RelationFlags diesel traits
//

impl std::convert::TryFrom<u32> for RelationFlags {
    type Error = failure::Error;
    fn try_from(relation_flags_raw: u32) -> Result<Self, Self::Error> {
        if relation_flags_raw > RelationFlags::ALL.bits {
            return Err(failure::format_err!("invalid RelationFlags value {:x}; expected a value in the range [0, {:x}]", relation_flags_raw, RelationFlags::ALL.bits));
        }
        Ok(RelationFlags { bits: relation_flags_raw })
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::sqlite::Sqlite> for RelationFlags {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        let relation_flags_raw = self.bits as i32;
        <i32 as diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::sqlite::Sqlite>>::to_sql(&relation_flags_raw, out)
    }
}

// TODO: Does there need to be FromSql<...> for RelationFlags?

impl<DB, ST> diesel::Queryable<ST, DB> for RelationFlags
where
    DB: diesel::backend::Backend,
    i32: diesel::Queryable<ST, DB>,
{
    type Row = <i32 as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        // TODO: Note that this will panic upon invalid value.  If this is a problem, then
        RelationFlags::try_from(i32::build(row) as u32).expect("invalid RelationFlags value; if this panicking is a problem, then some 'invalid' or 'unknown' enum variant should be added to RelationFlags")
    }
}
