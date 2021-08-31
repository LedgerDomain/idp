// This is private because the generated code is meant to be imported into the public modules listed above.
mod generated;

pub use crate::generated::idp::*;

impl From<&[u8]> for Sha256Sum {
    fn from(bytes: &[u8]) -> Self {
        Sha256Sum { value: bytes.to_vec() }
    }
}

impl From<Vec<u8>> for Sha256Sum {
    fn from(value: Vec<u8>) -> Self {
        Sha256Sum { value }
    }
}

impl From<Sha256Sum> for Seal {
    fn from(sha256sum: Sha256Sum) -> Self {
//         Seal { value: Some(seal::Value::Sha256Sum(sha256sum)) }
        Seal { sha256sum }
    }
}

impl From<&[u8]> for Nonce {
    fn from(bytes: &[u8]) -> Self {
        Nonce { value: bytes.to_vec() }
    }
}

impl From<&str> for Nonce {
    fn from(s: &str) -> Self {
        Nonce { value: s.as_bytes().to_vec() }
    }
}

impl From<Vec<u8>> for Nonce {
    fn from(value: Vec<u8>) -> Self {
        Nonce { value }
    }
}

impl From<&[u8]> for ContentType {
    fn from(bytes: &[u8]) -> Self {
        ContentType { value: bytes.to_vec() }
    }
}

impl From<Vec<u8>> for ContentType {
    fn from(value: Vec<u8>) -> Self {
        ContentType { value }
    }
}

impl From<&str> for ContentType {
    fn from(s: &str) -> Self {
        ContentType { value: s.as_bytes().to_vec() }
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Did { value: s.to_string() }
    }
}

impl From<String> for Did {
    fn from(value: String) -> Self {
        Did { value }
    }
}

impl UnixSeconds {
    pub fn now() -> UnixSeconds {
        UnixSeconds::from(chrono::Utc::now())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for UnixSeconds {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        UnixSeconds { value: dt.timestamp() }
    }
}

impl From<Seal> for PlumHeadSeal {
    fn from(value: Seal) -> Self {
        PlumHeadSeal { value }
    }
}

impl From<Seal> for PlumBodySeal {
    fn from(value: Seal) -> Self {
        PlumBodySeal { value }
    }
}

//
// PushRequest
//

impl From<PushHeadRequest> for PushRequest {
    fn from(value: PushHeadRequest) -> Self {
        PushRequest { value: Some(push_request::Value::PushHeadRequest(value)) }
    }
}

impl From<PushBodyRequest> for PushRequest {
    fn from(value: PushBodyRequest) -> Self {
        PushRequest { value: Some(push_request::Value::PushBodyRequest(value)) }
    }
}

impl From<PushHeadAndBodyRequest> for PushRequest {
    fn from(value: PushHeadAndBodyRequest) -> Self {
        PushRequest { value: Some(push_request::Value::PushHeadAndBodyRequest(value)) }
    }
}

//
// PushResponse
//

impl From<PushHeadResponse> for PushResponse {
    fn from(value: PushHeadResponse) -> Self {
        PushResponse { value: Some(push_response::Value::PushHeadResponse(value)) }
    }
}

impl From<PushBodyResponse> for PushResponse {
    fn from(value: PushBodyResponse) -> Self {
        PushResponse { value: Some(push_response::Value::PushBodyResponse(value)) }
    }
}

impl From<PushHeadAndBodyResponse> for PushResponse {
    fn from(value: PushHeadAndBodyResponse) -> Self {
        PushResponse { value: Some(push_response::Value::PushHeadAndBodyResponse(value)) }
    }
}

//
// PullRequest
//

impl From<PullHeadRequest> for PullRequest {
    fn from(value: PullHeadRequest) -> Self {
        PullRequest { value: Some(pull_request::Value::PullHeadRequest(value)) }
    }
}

impl From<PullBodyRequest> for PullRequest {
    fn from(value: PullBodyRequest) -> Self {
        PullRequest { value: Some(pull_request::Value::PullBodyRequest(value)) }
    }
}

impl From<PullHeadAndBodyRequest> for PullRequest {
    fn from(value: PullHeadAndBodyRequest) -> Self {
        PullRequest { value: Some(pull_request::Value::PullHeadAndBodyRequest(value)) }
    }
}

//
// PullResponse
//

impl From<PullHeadResponse> for PullResponse {
    fn from(value: PullHeadResponse) -> Self {
        PullResponse { value: Some(pull_response::Value::PullHeadResponse(value)) }
    }
}

impl From<PullBodyResponse> for PullResponse {
    fn from(value: PullBodyResponse) -> Self {
        PullResponse { value: Some(pull_response::Value::PullBodyResponse(value)) }
    }
}

impl From<PullHeadAndBodyResponse> for PullResponse {
    fn from(value: PullHeadAndBodyResponse) -> Self {
        PullResponse { value: Some(pull_response::Value::PullHeadAndBodyResponse(value)) }
    }
}

//
// DelRequest
//

impl From<DelHeadRequest> for DelRequest {
    fn from(value: DelHeadRequest) -> Self {
        DelRequest { value: Some(del_request::Value::DelHeadRequest(value)) }
    }
}

impl From<DelBodyRequest> for DelRequest {
    fn from(value: DelBodyRequest) -> Self {
        DelRequest { value: Some(del_request::Value::DelBodyRequest(value)) }
    }
}

impl From<DelHeadAndBodyRequest> for DelRequest {
    fn from(value: DelHeadAndBodyRequest) -> Self {
        DelRequest { value: Some(del_request::Value::DelHeadAndBodyRequest(value)) }
    }
}

//
// DelResponse
//

impl From<DelHeadResponse> for DelResponse {
    fn from(value: DelHeadResponse) -> Self {
        DelResponse { value: Some(del_response::Value::DelHeadResponse(value)) }
    }
}

impl From<DelBodyResponse> for DelResponse {
    fn from(value: DelBodyResponse) -> Self {
        DelResponse { value: Some(del_response::Value::DelBodyResponse(value)) }
    }
}

impl From<DelHeadAndBodyResponse> for DelResponse {
    fn from(value: DelHeadAndBodyResponse) -> Self {
        DelResponse { value: Some(del_response::Value::DelHeadAndBodyResponse(value)) }
    }
}

//
// Deriving Seal values from PlumHead and PlumBody
//

impl From<&PlumHead> for PlumHeadSeal {
    fn from(head: &PlumHead) -> PlumHeadSeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        hasher.update(&head.body_seal.value.sha256sum.value);

        hasher.update(head.body_length.to_le_bytes()); // Little-endian byte representation

        hasher.update(&head.body_content_type.value);

        if let Some(head_nonce) = &head.head_nonce_o {
            hasher.update(b"\x01");
            hasher.update(&head_nonce.value);
        } else {
            hasher.update(b"\x00");
        }

        if let Some(owner_did) = &head.owner_did_o {
            hasher.update(b"\x01");
            hasher.update(&owner_did.value);
        } else {
            hasher.update(b"\x00");
        }

        if let Some(created_at) = &head.created_at_o {
            hasher.update(b"\x01");
            hasher.update(created_at.value.to_le_bytes()); // Little-endian byte representation
        } else {
            hasher.update(b"\x00");
        }

        if let Some(metadata) = &head.metadata_o {
            hasher.update(b"\x01");
            hasher.update(metadata);
        } else {
            hasher.update(b"\x00");
        }

        PlumHeadSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().as_slice())))
    }
}

impl From<&PlumBody> for PlumBodySeal {
    fn from(body: &PlumBody) -> PlumBodySeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        // Computing the hash for the body is somewhat different.  Don't bother indicating
        // the presence/nonpresence of the nonce (as in PlumHeadSeal::from), but rather
        // just prepend the nonce if it's specified.  This is to make the PlumBodySeal
        // equal to the hash of body_content if there's no nonce.
        if let Some(body_nonce) = &body.body_nonce_o {
            hasher.update(&body_nonce.value);
        }

        hasher.update(&body.body_content);

        PlumBodySeal::from(Seal::from(Sha256Sum::from(hasher.finalize().as_slice())))
    }
}

//
// diesel trait implementations
//

//
// ContentType
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for ContentType {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.value, out)
    }
}

// TODO: Does there need to be FromSql<...> for ContentType?

impl<DB, ST> diesel::Queryable<ST, DB> for ContentType
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        ContentType::from(Vec::build(row))
    }
}

//
// Did
//

impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::sqlite::Sqlite> for Did {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <String as diesel::serialize::ToSql<diesel::sql_types::Text, diesel::sqlite::Sqlite>>::to_sql(&self.value, out)
    }
}

// TODO: Does there need to be FromSql<...> for Did?

impl<DB, ST> diesel::Queryable<ST, DB> for Did
where
    DB: diesel::backend::Backend,
    String: diesel::Queryable<ST, DB>,
{
    type Row = <String as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        Did::from(String::build(row))
    }
}

//
// Nonce
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for Nonce {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.value, out)
    }
}

// TODO: Does there need to be FromSql<...> for Nonce?

impl<DB, ST> diesel::Queryable<ST, DB> for Nonce
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        Nonce::from(Vec::build(row))
    }
}

//
// PlumBodySeal
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for PlumBodySeal {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.value.sha256sum.value, out)
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for PlumBodySeal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok(PlumBodySeal::from(Seal::from(Sha256Sum::from_sql(bytes)?)))
    }
}

impl<DB, ST> diesel::Queryable<ST, DB> for PlumBodySeal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        PlumBodySeal::from(Seal::from(Sha256Sum::from(Vec::build(row))))
    }
}

//
// PlumHeadSeal
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for PlumHeadSeal {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.value.sha256sum.value, out)
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for PlumHeadSeal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok(PlumHeadSeal::from(Seal::from(Sha256Sum::from_sql(bytes)?)))
    }
}

impl<DB, ST> diesel::Queryable<ST, DB> for PlumHeadSeal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        PlumHeadSeal::from(Seal::from(Sha256Sum::from(Vec::build(row))))
    }
}

//
// Seal
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for Seal {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.sha256sum.value, out)
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for Seal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok(Seal::from(Sha256Sum::from_sql(bytes)?))
    }
}

impl<DB, ST> diesel::Queryable<ST, DB> for Seal
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        Seal::from(Sha256Sum::build(row))
    }
}

// impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for Seal {
//     fn to_sql<W: std::io::Write>(
//         &self,
//         out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
//     ) -> diesel::serialize::Result {
//         <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(
//             match &self.value {
//                 Some(seal::Value::Sha256Sum(sha256sum)) => &sha256sum.value,
//                 None => {
//                     // return Err(Box::new(failure::format_err!("invalid Seal -- it was set to None")));
//                     panic!("invalid Seal -- it was set to None -- TODO: handle this error condition");
//                 }
//             },
//             out,
//         )
//     }
// }
//
// impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for Seal
// where
//     DB: diesel::backend::Backend,
//     Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
// {
//     fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
//         Ok(Seal::from(Sha256Sum::from_sql(bytes)?))
//     }
// }
//
// impl<DB, ST> diesel::Queryable<ST, DB> for Seal
// where
//     DB: diesel::backend::Backend,
//     Vec<u8>: diesel::Queryable<ST, DB>,
// {
//     type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
//     fn build(row: Self::Row) -> Self {
//         Seal::from(Sha256Sum::build(row))
//     }
// }

//
// Sha256Sum
//

impl diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite> for Sha256Sum {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <Vec<u8> as diesel::serialize::ToSql<diesel::sql_types::Binary, diesel::sqlite::Sqlite>>::to_sql(&self.value, out)
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for Sha256Sum
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok(Sha256Sum::from(Vec::from_sql(bytes)?))
    }
}

impl<DB, ST> diesel::Queryable<ST, DB> for Sha256Sum
where
    DB: diesel::backend::Backend,
    Vec<u8>: diesel::Queryable<ST, DB>,
{
    type Row = <Vec<u8> as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        Sha256Sum::from(Vec::build(row))
    }
}

//
// UnixSeconds
//

impl diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite> for UnixSeconds {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        <i64 as diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite>>::to_sql(
            &self.value,
            out,
        )
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB> for UnixSeconds
where
    DB: diesel::backend::Backend,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok(UnixSeconds { value: i64::from_sql(bytes)? })
    }
}

impl<DB, ST> diesel::Queryable<ST, DB> for UnixSeconds
where
    DB: diesel::backend::Backend,
    i64: diesel::Queryable<ST, DB>,
{
    type Row = <i64 as diesel::Queryable<ST, DB>>::Row;
    fn build(row: Self::Row) -> Self {
        UnixSeconds { value: i64::build(row) }
    }
}
