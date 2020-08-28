use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_serde_json::ReadJson;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

pub async fn handle_stream(mut stream: TcpStream) {
    let length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());
    let mut deserialized = ReadJson::<_, Value>::new(length_delimited);
}
