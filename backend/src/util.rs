use std::io::Cursor;

use anyhow::Result;
use minify_html::{minify, Cfg};
use once_cell::sync::Lazy;
use prost::Message;

pub fn protobuf_encode<T: Message>(message: T) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    buffer.reserve(message.encoded_len());
    message.encode(&mut buffer)?;
    Ok(buffer)
}

pub fn protobuf_decode<T>(message: &[u8]) -> Result<T>
where
    T: Message + Default,
{
    let message = T::decode(&mut Cursor::new(message))?;
    Ok(message)
}

pub fn minify_html(html: String) -> Result<String> {
    static CFG: Lazy<Cfg> = Lazy::new(|| Cfg::spec_compliant());
    let html = html.as_bytes();
    Ok(std::str::from_utf8(&minify(html, &CFG))?.to_owned())
}
