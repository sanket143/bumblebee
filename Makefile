run:
	cargo build && ./target/debug/bumblebee --project-path ./test-dir

js:
	node test-dir/index.js

test:
	cargo test -- --nocapture
