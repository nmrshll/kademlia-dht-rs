use crate::key::Key;
use crate::rout2::KnownNode;

pub const A_CONCURRENT_REQUESTS: usize = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FindValueResp {
    Nodes(Vec<KnownNode>),
    Value(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    Ping,
    FindNode(Vec<KnownNode>),
    FindValue(FindValueResp),
}

///////////////////
// TOKIO SERDE USING TOKIO CODECS
/////////////////

// TODO rm
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DummyData {
    field: i32,
    // pub hello: String,
}

use bytes::{BufMut, Bytes, BytesMut};
// use tokio::net::TcpStream;
use serde_json::Value;
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct MyCodec;
impl Decoder for MyCodec {
    type Item = DummyData; // (1)
    type Error = MyCodecErr; // (2)
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 1. If the stream hasn't provided enough bytes yet, or is empty,
        // the decoder should return Ok(None) to stop parsing frames
        let len = src.len();
        if len == 0 {
            return Ok(None);
        }

        // 2. If the frame is invalid,
        // consume all of the error bytes and return Err(MyCodecErr::ErrorKind) to report an Error frame to the consumer
        // if invalid {
        //     // src should be truncated to [next_start_index,len)
        //     src.split_to(next_start_index);
        //     return Err(MyCodecErr::NoImpl);
        // }

        // 3. If the bytes are valid, consume all the bytes in the current frame
        // and return Ok(Some(DummyData::EventKind))
        let v: Value = serde_json::from_reader(src.as_ref())?;
        dbg!(&v);
        let dd: DummyData = serde_json::from_reader(src.as_ref())?;
        dbg!(&dd);

        // src.split_to(next_start_index); // truncate src here too
        src.clear();
        return Ok(Some(dd));
    }
}

impl Encoder<DummyData> for MyCodec {
    type Error = MyCodecErr;

    fn encode(&mut self, data: DummyData, dest: &mut BytesMut) -> Result<(), MyCodecErr> {
        dest.put_uint(1u64, 64); // TODO replace this with real encoding
        Ok(())
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyCodecErr {
    #[error("MyCodec IoErr")]
    IoErr(#[from] io::Error),
    #[error("MyCodec SerdeErr")]
    SerdeErr(#[from] serde_json::Error),
    #[error("unknown MyCodec error")]
    Unknown,
}
impl PartialEq for MyCodecErr {
    fn eq(&self, other: &Self) -> bool {
        use std::error::Error;
        match (&self, &other) {
            (MyCodecErr::IoErr(a), MyCodecErr::IoErr(b)) => a.kind() == b.kind(),
            (MyCodecErr::SerdeErr(a), MyCodecErr::SerdeErr(b)) => {
                a.description() == b.description()
            }
            (MyCodecErr::Unknown, MyCodecErr::Unknown) => false,
            _ => false,
        }
    }
}

// pub type MyCodecConnection = Framed<TcpStream, MyCodec>; // (1)

impl MyCodec {
    pub fn new() -> Self {
        MyCodec {}
    }
    // pub async fn connect(addr: &SocketAddr) -> Result<MyCodecConnection, io::Error> {
    //     let tcp_stream = TcpStream::connect(addr).await?;
    //     Ok(MyCodec.framed(tcp_stream)) // (2)
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
    // and push the results to a Vec<Result<Option<DummyData>, MyCodecErr>>.
    // Here a custom consume function I wrote to test the output of my decoder.
    fn consume(codec: &mut MyCodec, bytes: &mut BytesMut) -> Vec<Result<DummyData, MyCodecErr>> {
        let mut result = Vec::new();
        loop {
            match codec.decode(bytes) {
                Ok(None) => {
                    break;
                }
                // output is Ok(some) or Err
                Ok(Some(dummyData)) => result.push(Ok(dummyData)),
                Err(myCodecErr) => result.push(Err(myCodecErr)),
            }
        }
        return result;
    }

    #[test]
    fn finished_message() {
        // This test validates that the codec converts the byte input
        // into DummyDatas correctly.
        // First, instantiate the codec.
        let mut codec = MyCodec::new();
        // Then create a BytesMut buffer from some bytes to be decoded.
        let mut bytes = BytesMut::from(b"{\"field\":3}".as_ref());
        // Finally consume the input bytes, and compare the frames that
        // that the decode function returns.
        let res_vec = consume(&mut codec, &mut bytes);

        // the bytes should be completely consumed, so `bytes.len()`
        // should be 0
        assert_eq!(bytes.len(), 0_usize);

        // Since we sent a message to the decoder that ends in "\r\n",
        // it should return a single telnet frame in the form of a
        // message event that contains the expected String value.
        let first_res: &Result<DummyData, MyCodecErr> = res_vec.first().unwrap();
        assert_eq!(*first_res, Ok(DummyData { field: 3 }));
    }

    // #[test]
    // fn message_encode() {
    //     // The encoder is responsible for turning DummyDatas into
    //     // byte frames. First, create the codec.
    //     let mut codec = MyCodec::new(4096);
    //     // Next, create a buffer to be written to.
    //     let mut output = BytesMut::new();
    //     // Finally, create the message event and encode the result.
    //     let message = DummyData::Message(String::from("Hello world!\r\n"));

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
