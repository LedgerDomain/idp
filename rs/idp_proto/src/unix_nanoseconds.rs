use crate::UnixNanoseconds;

impl UnixNanoseconds {
    pub fn now() -> UnixNanoseconds {
        UnixNanoseconds::from(chrono::Utc::now())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for UnixNanoseconds {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        UnixNanoseconds {
            value: dt.timestamp_nanos(),
        }
    }
}
