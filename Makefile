build:
	cargo build --release
build-x86_64:
	cargo build --release --target x86_64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/release
	cp -f target/x86_64-unknown-linux-gnu/release/ramwise target/release/release
	ls target
	ls target/release/
build-arm:
	cargo build --release --target aarch64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/release
	cp -f target/aarch64-unknown-linux-gnu/release/ramwise target/release/release
	ls target
	ls target/release/
build-mac:
	cargo build --release --target aarch64-apple-darwin
	mkdir -pv target
	mkdir -pv target/release
	cp -f target/aarch64-apple-darwin/release/ramwise target/release/release
	ls target
	ls target/release/
debug:
	cargo build
debug-x86_64:
	cargo build --target x86_64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/debug
	cp -f target/x86_64-unknown-linux-gnu/debug/ramwise target/release/debug
debug-arm:
	cargo build --target aarch64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/debug
	cp -f target/aarch64-unknown-linux-gnu/debug/ramwise target/release/debug
debug-mac:
	cargo build --target aarch64-apple-darwin
	mkdir -pv target
	mkdir -pv target/debug
	cp -f target/aarch64-apple-darwin/debug/ramwise target/release/debug
clean:
	cargo clean
install:
	mkdir -p $(DESTDIR)/usr/bin
	install -m 0755 target/release/ramwise $(DESTDIR)/usr/bin/ramwise
install-debug:
	mkdir -p $(DESTDIR)/usr/bin
	install -m 0755 target/debug/ramwise $(DESTDIR)/usr/bin/ramwise
