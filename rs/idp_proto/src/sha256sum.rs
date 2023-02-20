use crate::Sha256Sum;

impl AsRef<[u8]> for Sha256Sum {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

impl std::fmt::Display for Sha256Sum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for byte in &self.value {
            write!(f, "{:02X}", byte)?
        }
        Ok(())
    }
}
