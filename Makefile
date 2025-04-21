ifeq ($(OS), Windows_NT)
	EXE := .\target\debug\bumblebee.exe
else
	EXE := ./target/debug/bumblebee
endif

run:
	cargo build && $(EXE) --project-path ./test-dir

js:
	node test-dir/index.js

test:
	cargo test -- --nocapture
