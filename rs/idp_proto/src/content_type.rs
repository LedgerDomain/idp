use crate::ContentType;

impl AsRef<[u8]> for ContentType {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}
