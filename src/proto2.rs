use thiserror::Error;
//
use crate::key::Key;
use crate::rout2::KnownNode;
use crate::state2::StateErr;

pub const A_CONCURRENT_REQUESTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Store(Key, String), // TODO genericity
    FindNode(Key),
    FindValue(Key),
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub enum Reply {
    Ping,
    FindNode(Vec<KnownNode>),
    FindVal(FindValResp),
    #[from] // auto-implement From<ProtoErr>
    Err(ProtoErr),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FindValResp {
    Nodes(Vec<KnownNode>),
    Value(String),
}
impl From<Result<Reply, ProtoErr>> for Reply {
    fn from(res: Result<Reply, ProtoErr>) -> Self {
        res.unwrap_or_else(Reply::from)
    }
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ProtoErr {
    #[error("state err: {0}")]
    StateErr(#[from] StateErr),
    #[error("unknown Protocol error")]
    Unknown,
}

///////////////////
// TOKIO SERDE USING TOKIO CODECS
/////////////////

// // TODO rm
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct DummyData {
//     field: i32,
//     // pub hello: String,
// }

use bytes::{buf::ext::BufMutExt, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct ProtocolCodec;
// pub type ProtocolCodecConnection = Framed<TcpStream, ProtocolCodec>; // (1)
impl ProtocolCodec {
    pub fn new() -> Self {
        ProtocolCodec {}
    }
    // pub async fn connect(addr: &SocketAddr) -> Result<ProtocolCodecConnection, io::Error> {
    //     let tcp_stream = TcpStream::connect(addr).await?;
    //     Ok(ProtocolCodec.framed(tcp_stream)) // (2)
    // }
}
impl Decoder for ProtocolCodec {
    type Item = Request;
    type Error = CodecErr;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 1. If the stream hasn't provided enough bytes yet, or is empty,
        // the decoder should return Ok(None) to stop parsing frames
        let len = src.len();
        if len == 0 {
            return Ok(None);
        }

        // 2. If the frame is invalid,
        // consume all of the error bytes and return Err(CodecErr::ErrorKind) to report an Error frame to the consumer
        // if invalid {
        //     // src should be truncated to [next_start_index,len)
        //     src.split_to(next_start_index);
        //     return Err(CodecErr::NoImpl);
        // }

        // 3. If the bytes are valid, consume all bytes in the current frame into new BytesMut
        // and return Ok(Some(Request))
        let src_cp = src.split(); // empties src
        let req: Request = serde_json::from_reader(src_cp.as_ref())?;

        return Ok(Some(req));
    }
}
impl Encoder<Request> for ProtocolCodec {
    type Error = CodecErr;

    fn encode(&mut self, req: Request, dest: &mut BytesMut) -> Result<(), CodecErr> {
        serde_json::to_writer(dest.writer(), &req)?;
        Ok(())
    }
}

//////////////////////
// ERRORS
///////////////

#[derive(Error, Debug)]
pub enum CodecErr {
    #[error("ProtocolCodec IoErr: {0}")]
    IoErr(#[from] io::Error),
    #[error("ProtocolCodec SerdeErr: {0}")]
    SerdeErr(#[from] serde_json::Error),
    #[error("unknown ProtocolCodec error")]
    Unknown,
}
impl PartialEq for CodecErr {
    fn eq(&self, other: &Self) -> bool {
        use std::error::Error;
        match (&self, &other) {
            (CodecErr::IoErr(a), CodecErr::IoErr(b)) => a.kind() == b.kind(),
            (CodecErr::SerdeErr(a), CodecErr::SerdeErr(b)) => a.description() == b.description(),
            (CodecErr::Unknown, CodecErr::Unknown) => false,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_add() {
        assert_eq!(1 + 2, 3);
    }

    // To test a decoder, iterate over the output of each function call
    // and push the results to a Vec<Result<Option<Request>, CodecErr>>.
    // Here a custom consume function I wrote to test the output of my decoder.
    fn consume(codec: &mut ProtocolCodec, bytes: &mut BytesMut) -> Vec<Result<Request, CodecErr>> {
        let mut result = Vec::new();
        loop {
            match codec.decode(bytes) {
                Ok(None) => {
                    break;
                }
                Ok(Some(req)) => result.push(Ok(req)),
                Err(e) => result.push(Err(e)),
            }
        }
        return result;
    }

    #[test]
    fn msg_decode() {
        // Instantiate a Codec, and Bytes to be decoded into Request correctly
        let mut codec = ProtocolCodec::new();
        let mut bytes = BytesMut::from(b"\"Ping\"".as_ref());

        // Finally consume the input bytes, and compare the frames that
        // that the decode function returns.
        let res_vec = consume(&mut codec, &mut bytes);

        // the bytes should be completely consumed, so `bytes.len()`
        // should be 0
        assert_eq!(bytes.len(), 0_usize);

        // Since we sent a message to the decoder that ends in "\r\n", // not valid anymore
        // it should return a single Request frame in the form of a
        // message event that contains the expected value.
        let first_res: &Result<Request, CodecErr> = res_vec.first().unwrap();
        assert_eq!(*first_res, Ok(Request::Ping));
    }

    #[test]
    fn msg_encode() {
        // The encoder is responsible for turning a message (Request) into byte frames
        let mut codec = ProtocolCodec::new();
        let msg = Request::Ping;

        // Create a buffer to encode the message to
        let mut output = BytesMut::new();
        codec
            .encode(msg, &mut output)
            .expect("Invalid encoding sequence");

        // utf8 output
        assert_eq!(
            output,
            // The output should have the following bytes in the buffer
            Bytes::from(b"\"Ping\"".as_ref()),
        );
    }
}

// TOKIO CODEC BINCODE
// FramedRead upgrades TcpStream from an AsyncRead to a Stream
// type IOErrorStream = FramedRead<TcpStream, LengthDelimitedCodec>;
// stream::FromErr maps underlying IO errors into Bincode errors
// type BincodeErrStream = stream::FromErr<IOErrorStream, bincode::Error>;
// ReadBincode maps underlying bytes into Bincode-deserializable structs
// type BincodeStream = ReadBincode<BincodeErrStream, DummyData>;
