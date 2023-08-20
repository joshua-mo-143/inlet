test:
	cargo run -- y create --crud meme --auth --secrets && cd y && cargo clippy

retest:
	rm -rf y && make test