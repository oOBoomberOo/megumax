set shell := ["powershell.exe"]

examples args='':
	ls examples | foreach { cd "examples/$_"; echo $pwd; cargo run --release {{args}}; cd ../.. }

tests:
	cargo test --no-default-features
	cargo test --all-features