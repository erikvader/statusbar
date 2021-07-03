CONF := src/config.rs
SYSDHOOK := statusbar_suspend_$(USER)

.PHONY: all
all: $(CONF)
	cargo build --release
	mv target/release/statusbar .

$(CONF):
	cp config.def.rs $(CONF)

.PHONY: clean
clean:
	cargo clean
	rm -f statusbar

.PHONY: install-systemd
install-systemd:
ifeq ($(USER), root)
	$(error do not call as root, sudo is used in the appropriate place)
endif
	m4 -DM4USER=$(USER) -DM4FIFOECHO="$$(command -v fifoecho)" utils/statusbar_suspend > $(SYSDHOOK)
	chmod +x $(SYSDHOOK)
	sudo mv $(SYSDHOOK) /usr/lib/systemd/system-sleep
