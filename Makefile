default: build-release

build-release:
	@echo "Build release"
	@cargo build --release

test:
	@echo "Test release"
	@cargo test --release

conf:
	@echo "Copy core and libraries"
	@mkdir -p ${HOME}/.config/mal
	cp -f core/core.mal ${HOME}/.config/mal/
	@mkdir -p ${HOME}/.config/mal/libs
	cp -f libs/* ${HOME}/.config/mal/libs/

install: build-release test conf
	@echo "Install mal"
	sudo cp target/release/rust-mal /usr/local/bin/mal
	@sudo chown ${USER} /usr/local/bin/mal
	@echo "To start mal run:"
	@printf "\tmal [path/module.mal ...]\n\n"
