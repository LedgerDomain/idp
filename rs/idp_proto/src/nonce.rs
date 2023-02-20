use crate::Nonce;

impl AsRef<[u8]> for Nonce {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}
