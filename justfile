set shell := ["powershell.exe"]

examples args='':
	cargo build --release {{args}}
	ls examples | foreach { cd "examples/$_"; cargo run --release -q {{args}}; cd ../.. }

tests:
	cargo test --no-default-features
	cargo test --all-features