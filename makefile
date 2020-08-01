# EXAMPLES
ex.tokio:
	cd examples/tokio && \
	cargo run --example hello_tokio
ex.simple:
	RUST_LOG=trace cargo watch -x 'run --example simple'
ex.net2:
	cargo run --example net2
nc:
	nc 127.0.0.1 8080



# DEPS
export RUST_BACKTRACE=1