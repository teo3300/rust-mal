default: build-release

build-release:
	cargo build --release

test:
	cargo test --release

conf:
	mkdir -p ${HOME}/.config/mal
	cp -f core/core.mal ${HOME}/.config/mal/
	mkdir -p ${HOME}/.config/mal/libs
	cp -f libs/* ${HOME}/.config/mal/libs/

install: build-release conf
	sudo cp target/release/rust-mal /usr/local/bin/mal
	sudo chown ${USER} /usr/local/bin/mal
