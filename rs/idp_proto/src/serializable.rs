use crate::{Content, ContentClassifiable, ContentEncoding, ContentFormat, ContentMetadata};
use anyhow::Result;

/// Represents a type that can be serialized using a specified format.  This is necessary to implement the
/// process of serializing and encoding into Content, in which the serialized output is fed into a (possibly
/// empty) sequence of encodings.  See the function serialize_and_encode_to_content for that process, noting
/// that this is not a method of Serializable because of Rust's object safety rules for traits).  The function
/// serialize_using_serde_format will be a useful helper function for most structured types that impl this
/// trait, since they'll simply want to use serde serialization.
// TODO: Figure out how to indicate that there's an implied ContentFormat (e.g. for String, that's "charset=utf-8")
pub trait Serializable: ContentClassifiable {
    /// Serialize this content using the given format.
    fn serialize_using_format(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()>;
}

/// Impl of Contentifiable for common type String.
impl Serializable for String {
    fn serialize_using_format(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        self.as_str().serialize_using_format(content_format, writer)
    }
}

/// Impl of Contentifiable for common type &str.
// TODO: Is there a way to impl this for `str`?  I tried, but it didn't work.
impl Serializable for &str {
    fn serialize_using_format(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        // Having consts here seems kludgey, figure something better out later.
        const CHARSET_US_ASCII: &'static str = "charset=us-ascii";
        const CHARSET_UTF_8: &'static str = "charset=utf-8";
        match content_format.as_str() {
            CHARSET_US_ASCII => {
                anyhow::ensure!(self.is_ascii(), "couldn't serialize string using ContentFormat {:?} because the string contained non-ascii characters", CHARSET_US_ASCII);
                writer.write_all(self.as_bytes())?;
            }
            CHARSET_UTF_8 => {
                writer.write_all(self.as_bytes())?;
            }
            _ => {
                serialize_using_serde_format(self, content_format, writer)?;
            }
        }
        Ok(())
    }
}

/// Impl of Contentifiable for common type Vec<u8>.
impl Serializable for Vec<u8> {
    fn serialize_using_format(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        self.as_slice()
            .serialize_using_format(content_format, writer)
    }
}

/// Impl of Contentifiable for common type &[u8].
impl Serializable for &[u8] {
    fn serialize_using_format(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        match content_format.as_str() {
            "" => {
                // Just write the bytes raw.
                writer.write_all(self)?;
            }
            _ => {
                serialize_using_serde_format(self, content_format, writer)?;
            }
        }
        Ok(())
    }
}

/// Serialize this content using the given format and sequence of encodings into a writer.
fn serialize_and_encode<'a>(
    data: &dyn Serializable,
    writer: &mut dyn std::io::Write,
    content_format: &ContentFormat,
    codec_rev_i: &mut dyn std::iter::Iterator<Item = &'a str>,
) -> anyhow::Result<()> {
    // It's helpful to visualize a graph of the serialization and encoding process:
    //
    // Write:
    // data --format-> formatted-data --encoding1-> formatted-data-encoded-1 --encoding2-> formatted-data-encoded-1-2
    //
    // Because the terminal writer (which produces the fully-encoded data) is what we feed in via the `writer`
    // argument of this function, we have to construct the pipeline in reverse order, hence reversing the
    // content encoding iterator.

    match codec_rev_i.next() {
        Some(codec) => {
            // TODO: Eventually use a registration mechanism to register the codecs.
            match codec {
                "deflate" => {
                    #[cfg(feature = "encoding-deflate")]
                    {
                        log::debug!("encode_deflate begin");
                        let mut encoder = libflate::deflate::Encoder::new(writer);
                        serialize_and_encode(data, &mut encoder, content_format, codec_rev_i)?;
                        encoder.finish().into_result()?;
                        log::debug!("encode_deflate end");
                    }
                    #[cfg(not(feature = "encoding-deflate"))]
                    anyhow::bail!("Unsupported ContentEncoding codec {:?} (requires enabling the \"encoding-deflate\" crate feature)", codec);
                }
                "gzip" => {
                    #[cfg(feature = "encoding-gzip")]
                    {
                        log::debug!("encode_gzip begin");
                        let mut encoder = libflate::gzip::Encoder::new(writer)?;
                        serialize_and_encode(data, &mut encoder, content_format, codec_rev_i)?;
                        encoder.finish().into_result()?;
                        log::debug!("encode_gzip end");
                    }
                    #[cfg(not(feature = "encoding-gzip"))]
                    anyhow::bail!("Unsupported ContentEncoding codec {:?} (requires enabling the \"encoding-gzip\" crate feature)", codec);
                }
                "identity" | "" => {
                    // No encoding, so just recurse without creating an encoder.
                    serialize_and_encode(data, writer, content_format, codec_rev_i)?;
                }
                _ => {
                    anyhow::bail!("Unknown ContentEncoding codec {:?}", codec);
                }
            }
        }
        None => {
            // Base case of recursion.
            // We've constructed the whole pipe of encoders.  Serialize into the beginning of the pipe.
            data.serialize_using_format(content_format, writer)?;
        }
    }
    Ok(())
}

/// Serializes this content using the given format and sequence of encodings into a Content struct.
pub fn serialize_and_encode_to_content(
    data: &dyn Serializable,
    content_format: ContentFormat,
    mut content_encoding: ContentEncoding,
) -> Result<Content> {
    content_encoding.normalize();
    // Parse the comma-separated list of encodings into an iterator of individual codecs.
    // No need to trim whitespace because the normalize() call above already does that.
    let codec_i = content_encoding.as_str().split(',');
    let mut content_byte_v = Vec::new();
    serialize_and_encode(
        data,
        &mut content_byte_v,
        &content_format,
        &mut codec_i.rev(),
    )?;
    Ok(Content {
        content_metadata: ContentMetadata {
            content_length: content_byte_v.len() as u64,
            content_class: data.content_class(),
            content_format,
            content_encoding,
        },
        content_byte_v,
    })
}

/// Helper function which invokes the serde serializer matching the specified ContentFormat, or bails
/// with error if the ContentFormat doesn't correspond to a supported/known serde serializer.
pub fn serialize_using_serde_format<T: Serializable + serde::Serialize>(
    value: &T,
    content_format: &ContentFormat,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    match content_format.as_str() {
        "json" => {
            #[cfg(feature = "format-json")]
            serde_json::to_writer(writer, value)?;
            #[cfg(not(feature = "format-json"))]
            anyhow::bail!(
                "Unsupported ContentFormat {:?} (requires enabling the \"format-json\" crate feature)",
                content_format.as_str()
            );
        }
        "msgpack" => {
            #[cfg(feature = "format-msgpack")]
            rmp_serde::encode::write(writer, value)?;
            #[cfg(not(feature = "format-msgpack"))]
            anyhow::bail!(
                "Unsupported ContentFormat {:?} (requires enabling the \"format-msgpack\" crate feature)",
                content_format.as_str()
            );
        }
        _ => {
            anyhow::bail!("Unknown ContentFormat {:?}", content_format.as_str())
        }
    }
    Ok(())
}
