use crate::{
    Content, ContentClass, ContentClassifiable, ContentEncoding, ContentFormat, ContentMetadata,
};
use anyhow::Result;

/// Represents a type that can be turned into Content via a process of formatted serialization, followed
/// by a sequence of encodings.  The converse of this process is decoding, followed by deserialization.
/// Those processes are handled by the functions serialize_and_encode_to_content and
/// decode_and_deserialize_from_content (which are not part of this trait because of Rust's object safety
/// rules for traits).
// TODO: Figure out how to indicate that there's an implied ContentFormat (e.g. for String, that's "charset=utf-8")
pub trait Contentifiable: ContentClassifiable {
    fn content_class(&self) -> ContentClass {
        ContentClass::from(self.derive_content_class_str().to_string())
    }
    /// Serialize this content using the given format.
    // TODO: Use https://github.com/dtolnay/erased-serde instead, which apparently can easily handle
    // the serializer/deserializer format registration.
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()>;
}

/// Impl of Contentifiable for common type String.
impl Contentifiable for String {
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        self.as_str().serialize(content_format, writer)
    }
}

/// Impl of Contentifiable for common type &str.
// TODO: Is there a way to impl this for `str`?  I tried, but it didn't work.
impl Contentifiable for &str {
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        // Having consts here seems kludgey, figure something better out later.
        const CHARSET_US_ASCII: &'static str = "charset=us-ascii";
        const CHARSET_UTF_8: &'static str = "charset=utf-8";
        #[cfg(feature = "format-json")]
        const JSON: &'static str = "json";
        #[cfg(feature = "format-msgpack")]
        const MSGPACK: &'static str = "msgpack";
        match content_format.as_str() {
            CHARSET_US_ASCII => {
                anyhow::ensure!(self.is_ascii(), "couldn't serialize string using ContentFormat {:?} because the string contained non-ascii characters", CHARSET_US_ASCII);
                writer.write_all(self.as_bytes())?;
            }
            CHARSET_UTF_8 => {
                writer.write_all(self.as_bytes())?;
            }
            #[cfg(feature = "format-json")]
            JSON => {
                serde_json::to_writer(writer, self)?;
            }
            #[cfg(feature = "format-msgpack")]
            MSGPACK => {
                rmp_serde::encode::write(writer, self)?;
            }
            s => {
                anyhow::bail!("Unsupported String ContentFormat {:?}", s)
            }
        }
        Ok(())
    }
}

/// Impl of Contentifiable for common type Vec<u8>.
impl Contentifiable for Vec<u8> {
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        self.as_slice().serialize(content_format, writer)
    }
}

/// Impl of Contentifiable for common type &[u8].
impl Contentifiable for &[u8] {
    fn serialize(
        &self,
        content_format: &ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        match content_format.as_str() {
            #[cfg(feature = "format-json")]
            "json" => Ok(serde_json::to_writer(writer, self)?),
            #[cfg(feature = "format-msgpack")]
            "msgpack" => Ok(rmp_serde::encode::write(writer, self)?),
            _ => {
                // Just write the bytes raw.
                Ok(writer.write_all(self)?)
            }
        }
    }
}

/// Serialize this content using the given format and sequence of encodings into a writer.
fn serialize_and_encode<'a>(
    data: &dyn Contentifiable,
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
                #[cfg(feature = "encoding-deflate")]
                "deflate" => {
                    log::debug!("encode_deflate begin");
                    let mut encoder = libflate::deflate::Encoder::new(writer);
                    serialize_and_encode(data, &mut encoder, content_format, codec_rev_i)?;
                    encoder.finish().into_result()?;
                    log::debug!("encode_deflate end");
                }
                #[cfg(feature = "encoding-gzip")]
                "gzip" => {
                    log::debug!("encode_gzip begin");
                    let mut encoder = libflate::gzip::Encoder::new(writer)?;
                    serialize_and_encode(data, &mut encoder, content_format, codec_rev_i)?;
                    encoder.finish().into_result()?;
                    log::debug!("encode_gzip end");
                }
                "identity" | "" => {
                    // No encoding, so just recurse without creating an encoder.
                    serialize_and_encode(data, writer, content_format, codec_rev_i)?;
                }
                _ => {
                    // NOTE: If you hit this, you may have just forgotten to enable one of the encoding-* features.
                    anyhow::bail!("Unsupported ContentEncoding codec {:?}", codec);
                }
            }
        }
        None => {
            // Base case of recursion.
            // We've constructed the whole pipe of encoders.  Serialize into the beginning of the pipe.
            data.serialize(content_format, writer)?;
        }
    }
    Ok(())
}

/// Serializes this content using the given format and sequence of encodings into a Content struct.
pub fn serialize_and_encode_to_content(
    data: &dyn Contentifiable,
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

/// Deserialize from a reader into the given type using the given format and sequence of encodings.
fn decode_and_deserialize<'a, T: Contentifiable + serde::de::DeserializeOwned>(
    reader: &mut dyn std::io::Read,
    content_format: &ContentFormat,
    codec_i: &mut dyn std::iter::DoubleEndedIterator<Item = &'a str>,
) -> Result<T> {
    // It's helpful to visualize a graph of the serialization and encoding process:
    //
    // Read:
    // formatted-data-encoded-1-2 --encoding2-> formatted-data-encoded-1 --encoding1-> formatted-data --format-> data
    //
    // Because the terminal reader (which reads the fully-encoded data) is what we feed in via the `reader`
    // argument of this function, we have to construct the pipeline in reverse order, hence reversing the
    // content encoding iterator.

    log::trace!(
        "decode_and_deserialize; content_format: {:?}",
        content_format.as_str()
    );
    #[allow(unused_mut)]
    let mut r: Box<dyn std::io::Read> = Box::new(reader);
    for codec in codec_i.rev() {
        log::trace!("decode_and_deserialize; codec: {:?}", codec);
        match codec {
            #[cfg(feature = "encoding-deflate")]
            "deflate" => {
                r = Box::new(libflate::deflate::Decoder::new(r));
            }
            #[cfg(feature = "encoding-gzip")]
            "gzip" => {
                r = Box::new(libflate::gzip::Decoder::new(r)?);
            }
            "identity" | "" => {
                // No transformation.
            }
            _ => {
                let _ = r;
                // NOTE: If you hit this, you may have just forgotten to enable one of the encoding-* features.
                anyhow::bail!("Unsupported ContentEncoding codec {:?}", codec);
            }
        }
    }
    match content_format.as_str() {
        #[cfg(feature = "format-json")]
        "json" => Ok(serde_json::from_reader(r)?),
        #[cfg(feature = "format-msgpack")]
        "msgpack" => Ok(rmp_serde::decode::from_read(r)?),
        _ => {
            // NOTE: If you hit this, you may have just forgotten to enable one of the format-* features.
            anyhow::bail!("Unsupported ContentFormat: {:?}", content_format.as_str());
        }
    }
}

/// Deserializes using the given format and sequence of encodings from a Content struct.
pub fn decode_and_deserialize_from_content<
    T: ContentClassifiable + Contentifiable + serde::de::DeserializeOwned,
>(
    content: &Content,
) -> Result<T> {
    log::debug!(
        "decode_and_deserialize_from_content; content_format: {:?}, content_encoding: {:?}",
        content.content_metadata.content_format.as_str(),
        content.content_metadata.content_encoding
    );
    // TODO: Have a real error type and return specific errors like ContentClass mismatch, decode error, deserialization error, etc.
    anyhow::ensure!(
        content.content_metadata.content_class.as_str() == T::content_class_str(),
        "ContentClass mismatch"
    );
    let mut codec_i = content
        .content_metadata
        .content_encoding
        .as_str()
        .split(',')
        .map(|s| s.trim());
    decode_and_deserialize(
        &mut content.content_byte_v.as_slice(),
        &content.content_metadata.content_format,
        &mut codec_i,
    )
}
