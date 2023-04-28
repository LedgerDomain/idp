use crate::{Hashable, UnixNanoseconds};

impl UnixNanoseconds {
    pub fn now() -> Self {
        Self::from(chrono::Utc::now())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for UnixNanoseconds {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self::from(dt.timestamp_nanos())
    }
}

impl From<UnixNanoseconds> for chrono::DateTime<chrono::Utc> {
    fn from(unix_nanoseconds: UnixNanoseconds) -> Self {
        Self::from_utc(
            chrono::NaiveDateTime::from_timestamp(
                unix_nanoseconds.value / 1_000_000_000,
                (unix_nanoseconds.value % 1_000_000_000) as u32,
            ),
            chrono::Utc,
        )
    }
}

impl Hashable for UnixNanoseconds {
    /// Explicitly use little-endian byte order.
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.to_le_bytes().update_hasher(hasher);
    }
}
