all:
	cargo build 

test:
	cargo test --color=always --lib membrane::test::test_wasm --no-fail-fast --manifest-path ${PWD}/Cargo.toml --  --exact -Z unstable-options --show-output

clean:
	cargo clean

