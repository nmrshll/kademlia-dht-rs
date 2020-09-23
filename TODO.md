# TODO

- respond on the stream
- make request contain source Node (or just key and get addr from tcp/quic)
- client rpc and rust API for Kad
- networking: Request instead of DummyData
  - fix tests for Request Codec
  - figure out streaming several Requests with Codec
- networking: transition to Quic
- networking: prost protobufs instead of JSON ?

## DONE

- start Request handlers
- error handling in proto::Reply
