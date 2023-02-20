use crate::Seal;

impl AsRef<[u8]> for Seal {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

impl std::fmt::Display for Seal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.sha256sum)
    }
}
