build:
	cargo build --release
build-x86_64:
	cargo build --release --target x86_64-unknown-linux-gnu
build-arm:
	cargo build --release --target aarch64-unknown-linux-gnu
build-mac:
	cargo build --release --target aarch64-apple-darwin
debug:
	cargo build
debug-x86_64:
	cargo build --target x86_64-unknown-linux-gnu
debug-arm:
	cargo build --target aarch64-unknown-linux-gnu
debug-mac:
	cargo build --target aarch64-apple-darwin
clean:
	cargo clean
install:
	mkdir -p $(DESTDIR)/usr/bin
	install -m 0755 target/release/ramwise $(DESTDIR)/usr/bin/ramwise
