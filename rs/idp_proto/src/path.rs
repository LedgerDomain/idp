use crate::Path;

impl Path {
    pub fn validate(&self) -> Result<(), String> {
        if self.is_empty() {
            return Err("a Path is not allowed to be empty".to_string());
        }
        if self.contains('/') {
            return Err("a Path is not allowed to contain the '/' char".to_string());
        }
        Ok(())
    }
}
