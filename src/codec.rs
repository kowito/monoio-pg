use bytes::BytesMut;
use monoio_codec::{Decoder, Encoder};
use postgres_protocol::message::backend;
use crate::error::{Error, Result};

pub struct PostgresCodec;

impl Decoder for PostgresCodec {
    type Item = backend::Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<monoio_codec::Decoded<Self::Item>> {
        if src.len() < 5 {
            return Ok(monoio_codec::Decoded::Insufficient);
        }

        let _tag = src[0];
        let len = i32::from_be_bytes([src[1], src[2], src[3], src[4]]) as usize;

        if src.len() < len + 1 {
            return Ok(monoio_codec::Decoded::Insufficient);
        }

        let mut data = src.split_to(len + 1);
        match backend::Message::parse(&mut data) {
            Ok(Some(msg)) => Ok(monoio_codec::Decoded::Some(msg)),
            Ok(None) => Ok(monoio_codec::Decoded::Insufficient),
            Err(e) => Err(Error::Protocol(e.to_string())),
        }
    }
}

impl Encoder<BytesMut> for PostgresCodec {
    type Error = Error;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<()> {
        dst.extend_from_slice(&item);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_codec_decode_ready_for_query() {
        let mut codec = PostgresCodec;
        let mut src = BytesMut::from(&b"Z\x00\x00\x00\x05I"[..]);
        let msg = codec.decode(&mut src).unwrap();
        match msg {
            monoio_codec::Decoded::Some(backend::Message::ReadyForQuery(_)) => {}
            _ => panic!("Expected ReadyForQuery"),
        }
    }

    #[test]
    fn test_codec_decode_incomplete() {
        let mut codec = PostgresCodec;
        let mut src = BytesMut::from(&b"Z\x00\x00\x00"[..]);
        let msg = codec.decode(&mut src).unwrap();
        assert!(matches!(msg, monoio_codec::Decoded::Insufficient));
    }
}
