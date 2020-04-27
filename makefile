CONF := src/config.rs

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
