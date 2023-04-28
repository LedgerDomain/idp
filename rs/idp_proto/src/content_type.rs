use crate::{ContentClass, ContentFormat, ContentType, Hashable};
use anyhow::Result;

impl ContentType {
    /// This is rather experimental.  The decoupling of HTTP ContentType into distinct ContentClass and
    /// ContentFormat is quite irregular and not necessarily well-defined.
    /// Handle the exceptional ones that are organically inherited from HTTP content-type header standards.
    /// Then handle the ones that actually decouple properly.  This is not exhaustive, and is only meant to
    /// be a starting point.
    /// References:
    /// - https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type
    /// - https://www.iana.org/assignments/media-types/media-types.xhtml
    pub fn derive_from(
        content_class: &ContentClass,
        content_format: &ContentFormat,
    ) -> Result<Self> {
        match content_class.as_str() {
            "audio" | "font" | "image" | "video" => Ok(ContentType::from(format!(
                "{}/{}",
                content_class.as_str(),
                content_format.as_str()
            ))),
            content_class_str if content_class_str.starts_with("application/") => {
                if content_format.is_empty() {
                    // Leave blank.  E.g. "application/json".
                    Ok(ContentType::from(content_class_str.to_string()))
                } else {
                    // The "application" content types typically use '+' to adjoin the format.
                    Ok(ContentType::from(format!(
                        "{}+{}",
                        content_class_str,
                        content_format.as_str()
                    )))
                }
            }
            content_class_str if content_class_str.starts_with("text/") => {
                if content_format.is_empty() {
                    // Leave blank.  E.g. "text/plain".
                    // References:
                    // - https://stackoverflow.com/questions/49552112/is-the-charset-component-mandatory-in-the-http-content-type-header
                    Ok(ContentType::from(content_class_str.to_string()))
                } else {
                    // The "text" content types typically use ';' to adjoin the format (i.e. charset).
                    Ok(ContentType::from(format!(
                        "{};{}",
                        content_class_str,
                        content_format.as_str()
                    )))
                }
            }
            content_class_str => {
                anyhow::bail!("Unsupported content class: {}", content_class_str);
            }
        }
    }
}

impl Hashable for ContentType {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
