up:
	cargo run -- create --crud meme --auth --secrets --name y

down:
	rm -rf y 

test:
	make up && cd y && cargo clippy

retest:
	make down && make test