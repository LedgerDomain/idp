use crate::{ContentClass, Hashable};

impl ContentClass {
    pub fn text_plain() -> Self {
        Self::from("text/plain".to_string())
    }
    pub fn application_octet_stream() -> Self {
        Self::from("application/octet-stream".to_string())
    }
    pub fn audio() -> Self {
        Self::from("audio".to_string())
    }
    pub fn image() -> Self {
        Self::from("image".to_string())
    }
    pub fn video() -> Self {
        Self::from("video".to_string())
    }
    pub fn font() -> Self {
        Self::from("font".to_string())
    }
}

impl Hashable for ContentClass {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
