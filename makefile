# EXAMPLES
ex.tokio:
	cd examples/tokio && \
	cargo run --example hello_tokio
ex.simple:
	RUST_LOG=trace cargo watch -x 'run --example simple'
ex.kad2:
	cargo run --example kad2
nc:
	nc 0.0.0.0 8908



# DEPS
export RUST_BACKTRACE=1