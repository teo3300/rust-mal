default: build-release

build-release:
	cargo build --release

test:
	cargo test --release

install: build-release
	mkdir -p ${HOME}/.config/mal
	cp -f core/core.mal ${HOME}/.config/mal/
	sudo cp target/release/rust-mal /usr/local/bin/mal
	sudo chown ${USER} /usr/local/bin/mal
