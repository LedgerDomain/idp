use crate::{Content, ContentClassifiable, ContentFormat};
use anyhow::Result;

/// Represents a type that can be deserialized using a specified format.  This is necessary to implement the
/// process of decoding from Content and deserializing.  See the function decode_and_deserialize_from_content
/// for that process.  The function deserialize_using_serde_format will be a useful helper function for most
/// structured types that impl this trait, since they'll simply want to use serde deserialization.
pub trait Deserializable: ContentClassifiable + Sized {
    fn deserialize_using_format(
        content_format: &ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self>;
}

impl Deserializable for String {
    fn deserialize_using_format(
        content_format: &ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        // Kludgey -- collect these into a single place.
        const CHARSET_US_ASCII: &'static str = "charset=us-ascii";
        const CHARSET_UTF_8: &'static str = "charset=utf-8";
        match content_format.as_str() {
            CHARSET_US_ASCII => {
                let mut buf = String::new();
                reader.read_to_string(&mut buf)?;
                anyhow::ensure!(buf.is_ascii(), "ContentFormat was {:?} but there were non-ASCII characters in the serialized data", CHARSET_US_ASCII);
                Ok(buf)
            }
            CHARSET_UTF_8 => {
                let mut buf = String::new();
                reader.read_to_string(&mut buf)?;
                Ok(buf)
            }
            _ => Ok(deserialize_using_serde_format(content_format, reader)?),
        }
    }
}

impl Deserializable for Vec<u8> {
    fn deserialize_using_format(
        content_format: &ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        match content_format.as_str() {
            "" => {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                Ok(buf)
            }
            _ => Ok(deserialize_using_serde_format(content_format, reader)?),
        }
    }
}

pub fn deserialize_using_serde_format<T: serde::de::DeserializeOwned>(
    content_format: &ContentFormat,
    reader: &mut dyn std::io::Read,
) -> Result<T> {
    match content_format.as_str() {
        "json" => {
            #[cfg(feature = "format-json")]
            {
                Ok(serde_json::from_reader(reader)?)
            }
            #[cfg(not(feature = "format-json"))]
            anyhow::bail!("Unsupported ContentFormat {:?} (requires enabling the \"format-json\" crate feature)", content_format.as_str());
        }
        "msgpack" => {
            #[cfg(feature = "format-msgpack")]
            {
                Ok(rmp_serde::decode::from_read(reader)?)
            }
            #[cfg(not(feature = "format-msgpack"))]
            anyhow::bail!("Unsupported ContentFormat {:?} (requires enabling the \"format-msgpack\" crate feature)", content_format.as_str());
        }
        _ => {
            let _ = reader;
            anyhow::bail!("Unknown ContentFormat {:?}", content_format.as_str());
        }
    }
}

/// Deserialize from a reader into the given type using the given format and sequence of encodings.
// fn decode_and_deserialize<'a, T: Serializable + serde::de::DeserializeOwned>(
fn decode_and_deserialize<'a, T: Deserializable>(
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
            "deflate" => {
                #[cfg(feature = "encoding-deflate")]
                {
                    r = Box::new(libflate::deflate::Decoder::new(r));
                }
                #[cfg(not(feature = "encoding-deflate"))]
                anyhow::bail!("Unsupported ContentEncoding codec {:?} (requires enabling the \"encoding-deflate\" crate feature)", codec);
            }
            "gzip" => {
                #[cfg(feature = "encoding-gzip")]
                {
                    r = Box::new(libflate::gzip::Decoder::new(r)?);
                }
                #[cfg(not(feature = "encoding-gzip"))]
                anyhow::bail!("Unsupported ContentEncoding codec {:?} (requires enabling the \"encoding-deflate\" crate feature)", codec);
            }
            "identity" | "" => {
                // No transformation.
            }
            _ => {
                let _ = r;
                anyhow::bail!("Unknown ContentEncoding codec {:?}", codec);
            }
        }
    }
    T::deserialize_using_format(content_format, r.as_mut())
}

/// Deserializes using the given format and sequence of encodings from a Content struct.
pub fn decode_and_deserialize_from_content<T: Deserializable>(content: &Content) -> Result<T> {
    log::trace!(
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
