MAL_HOME ?= ${HOME}/.config/mal
BINARY_DIR ?= /usr/local/bin
CONFIG_FILE := ${MAL_HOME}/config.mal

default: build-release

build-release: test
	@echo "Building release for installation"
	@cargo build --release -q

test:
	@echo "Testing release"
	@MAL_HOME=core cargo test --release -q

conf: test
	@echo "Copying config"
	@mkdir -p ${MAL_HOME}
	@cp -f core/core.mal ${MAL_HOME}/
	@mkdir -p ${MAL_HOME}/libs
	@cp -f libs/* ${MAL_HOME}/libs/
	@touch ${MAL_HOME}/.mal-history
	@touch ${MAL_HOME}/config.mal
	@test -s ${CONFIG_FILE} || (\
		echo ";; Write here your mal config" >> ${MAL_HOME}/config.mal\
		&& echo "; (def! BANNER \"\") ; Hide banner at startup" >> ${MAL_HOME}/config.mal\
		&& echo '' >> ${MAL_HOME}/config.mal\
		&& echo '(def! BANNER (str BANNER' >> ${MAL_HOME}/config.mal\
	    && echo '";\n; **** To remove this banner and config mal, edit: ****\n"' >> ${MAL_HOME}/config.mal\
		&& echo '";     /Users/rogora/.config/mal/config.mal\n"))' >> ${MAL_HOME}/config.mal\
	)

install: build-release test conf
	@echo "Installing release"
	@echo "Provide password for installing \"mal\" to \"${BINARY_DIR}\""
	@sudo mkdir -p ${BINARY_DIR}
	@sudo cp target/release/rust-mal ${BINARY_DIR}/mal
	@sudo chown ${USER} ${BINARY_DIR}/mal
	@echo
	@echo '***************************************'
	@echo '* mal has been successfully installed *'
	@echo '***************************************'
	@echo IMPORTANT NOTES:
	@echo
	@echo "Make sure that \"${BINARY_DIR}\" is included into \x24PATH"
	@echo "To start mal run:"
	@printf "\tmal [path/to/script [args ...]]\n\n"
	@echo "To config mal edit:"
	@printf "\t${MAL_HOME}/config.mal\n"
