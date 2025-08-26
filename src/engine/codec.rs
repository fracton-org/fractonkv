use crate::engine::MAX_REQ_BYTES;
use bytes::BytesMut;
use redis_protocol::resp2::types::BytesFrame;
use redis_protocol::resp2::{decode, encode};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct RedisCodec;

impl RedisCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for RedisCodec {
    type Item = BytesFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesFrame>, io::Error> {
        if src.len() > MAX_REQ_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request too large",
            ));
        }

        if src.is_empty() {
            return Ok(None);
        }

        match decode::decode_bytes_mut(src) {
            Ok(Some((frame, _, _))) => {
                // src.advance(consumed);
                Ok(Some(frame))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }
}

impl Encoder<BytesFrame> for RedisCodec {
    type Error = io::Error;

    fn encode(&mut self, item: BytesFrame, dst: &mut BytesMut) -> Result<(), io::Error> {
        encode::extend_encode(dst, &item, false)
            .map(|_| ())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("encode error: {:?}", e)))
    }
}
