use crate::key::Key;
use crate::rout2::KnownNode;

pub const A_CONCURRENT_REQUESTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// #[serde(tag = "type")]
pub enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}
// pub struct StoreReq {
//     Key: String,
//     Value: String,
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    Ping,
    FindNode(Vec<KnownNode>),
    FindValue(FindValueResp),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FindValueResp {
    Nodes(Vec<KnownNode>),
    Value(String),
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

use bytes::{BufMut, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct ProtocolCodec;
impl Decoder for ProtocolCodec {
    type Item = Request;
    type Error = ProtocolErr;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 1. If the stream hasn't provided enough bytes yet, or is empty,
        // the decoder should return Ok(None) to stop parsing frames
        let len = src.len();
        if len == 0 {
            return Ok(None);
        }

        // 2. If the frame is invalid,
        // consume all of the error bytes and return Err(ProtocolErr::ErrorKind) to report an Error frame to the consumer
        // if invalid {
        //     // src should be truncated to [next_start_index,len)
        //     src.split_to(next_start_index);
        //     return Err(ProtocolErr::NoImpl);
        // }

        // 3. If the bytes are valid, consume all bytes in the current frame into new BytesMut
        // and return Ok(Some(Request))
        let src_cp = src.split(); // empties src
        let req: Request = serde_json::from_reader(src_cp.as_ref())?;

        return Ok(Some(req));
    }
}

impl Encoder<Request> for ProtocolCodec {
    type Error = ProtocolErr;

    fn encode(&mut self, _data: Request, dest: &mut BytesMut) -> Result<(), ProtocolErr> {
        dest.put_uint(1u64, 64); // TODO replace this with real encoding
        Ok(())
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolErr {
    #[error("ProtocolCodec IoErr: {0}")]
    IoErr(#[from] io::Error),
    #[error("ProtocolCodec SerdeErr: {0}")]
    SerdeErr(#[from] serde_json::Error),
    #[error("unknown ProtocolCodec error")]
    Unknown,
}
impl PartialEq for ProtocolErr {
    fn eq(&self, other: &Self) -> bool {
        use std::error::Error;
        match (&self, &other) {
            (ProtocolErr::IoErr(a), ProtocolErr::IoErr(b)) => a.kind() == b.kind(),
            (ProtocolErr::SerdeErr(a), ProtocolErr::SerdeErr(b)) => {
                a.description() == b.description()
            }
            (ProtocolErr::Unknown, ProtocolErr::Unknown) => false,
            _ => false,
        }
    }
}

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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(1 + 2, 3);
    }

    // To test a decoder, iterate over the output of each function call
    // and push the results to a Vec<Result<Option<Request>, ProtocolErr>>.
    // Here a custom consume function I wrote to test the output of my decoder.
    fn consume(
        codec: &mut ProtocolCodec,
        bytes: &mut BytesMut,
    ) -> Vec<Result<Request, ProtocolErr>> {
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
    fn finished_message() {
        // This test validates that the codec converts the byte input
        // into Requests correctly.
        // First, instantiate the codec.
        let mut codec = ProtocolCodec::new();
        // Then create a BytesMut buffer from some bytes to be decoded.
        let mut bytes = BytesMut::from(b"{\"field\":3}".as_ref()); // TOMORROW FIX TEST
                                                                   // Finally consume the input bytes, and compare the frames that
                                                                   // that the decode function returns.
        let res_vec = consume(&mut codec, &mut bytes);

        // the bytes should be completely consumed, so `bytes.len()`
        // should be 0
        assert_eq!(bytes.len(), 0_usize);

        // Since we sent a message to the decoder that ends in "\r\n",
        // it should return a single telnet frame in the form of a
        // message event that contains the expected String value.
        let first_res: &Result<Request, ProtocolErr> = res_vec.first().unwrap();
        // assert_eq!(*first_res, Ok(DummyData { field: 3 }));
        assert_eq!(*first_res, Ok(Request::Ping));
    }

    // #[test]
    // fn message_encode() {
    //     // The encoder is responsible for turning Requests into
    //     // byte frames. First, create the codec.
    //     let mut codec = ProtocolCodec::new(4096);
    //     // Next, create a buffer to be written to.
    //     let mut output = BytesMut::new();
    //     // Finally, create the message event and encode the result.
    //     let message = Request::Message(String::from("Hello world!\r\n"));

    //     // encode the message and read the output
    //     codec.encode(message, &mut output).expect("Invalid encoding sequence");

    //     // utf8 output
    //     assert_eq!(
    //         output,
    //         // The output should have the following bytes in the buffer
    //         BytesMut::from(vec![
    //             0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x0d, 0x0a,
    //         ]),
    //     );
    // }
}