default: build-release

build-release:
	@echo "Build release"
	@cargo build --release

test:
	@echo "Test release"
	@MAL_HOME=core cargo test --release

conf: test
	@echo "Copy core and libraries"
	@mkdir -p ${HOME}/.config/mal
	@touch ${HOME}/.config/mal/.mal-history
	cp -f core/core.mal ${HOME}/.config/mal/
	@mkdir -p ${HOME}/.config/mal/libs
	cp -f libs/* ${HOME}/.config/mal/libs/

install: build-release test conf
	@echo "Install mal"
	@sudo mkdir -p /usr/local/bin
	sudo cp target/release/rust-mal /usr/local/bin/mal
	@sudo chown ${USER} /usr/local/bin/mal
	@echo "To start mal run:"
	@printf "\tmal [path/module.mal ...]\n\n"
