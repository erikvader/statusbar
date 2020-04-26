.PHONY: all
all:
	cargo build --release
	mv target/release/statusbar .

.PHONY: clean
clean:
	cargo clean
	rm -f statusbar
