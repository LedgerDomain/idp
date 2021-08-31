table! {
    plum_bodies (body_seal) {
        body_seal -> Binary,
        row_inserted_at -> BigInt,
        body_nonce_o -> Nullable<Binary>,
        body_content_o -> Nullable<Binary>,
    }
}

table! {
    plum_heads (head_seal) {
        head_seal -> Binary,
        row_inserted_at -> BigInt,
        body_seal -> Binary,
        body_length -> BigInt,
        body_content_type -> Binary,
        head_nonce_o -> Nullable<Binary>,
        owner_did_o -> Nullable<Text>,
        created_at_o -> Nullable<BigInt>,
        metadata_o -> Nullable<Binary>,
    }
}

allow_tables_to_appear_in_same_query!(
    plum_bodies,
    plum_heads,
);
