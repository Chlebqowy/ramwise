build:
	cargo build --release
build-x86_64:
	cargo build --release --target x86_64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/release
	cp -f target/x86_64-unknown-linux-gnu/release/ramwise target/release/ramwise
build-aarch64:
	cargo build --release --target aarch64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/release
	cp -f target/aarch64-unknown-linux-gnu/release/ramwise target/release/ramwise
debug:
	cargo build
debug-x86_64:
	cargo build --target x86_64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/debug
	cp -f target/x86_64-unknown-linux-gnu/debug/ramwise target/debug/ramwise
debug-aarch64:
	cargo build --target aarch64-unknown-linux-gnu
	mkdir -pv target
	mkdir -pv target/debug
	cp -f target/aarch64-unknown-linux-gnu/debug/ramwise target/debug/ramwise
clean:
	cargo clean
install:
	mkdir -pv $(DESTDIR)/usr
	mkdir -pv $(DESTDIR)/usr/bin
	install -m 0755 target/release/ramwise $(DESTDIR)/usr/bin/ramwise
	mkdir -pv $(DESTDIR)/usr/share
	mkdir -pv $(DESTDIR)/usr/share/licenses/
	mkdir -pv $(DESTDIR)/usr/share/licenses/ramwise/
	install -m 0644 LICENSE "$(DESTDIR)/usr/share/licenses/ramwise/LICENSE"
install-debug:
	mkdir -pv $(DESTDIR)/usr
	mkdir -pv $(DESTDIR)/usr/bin
	install -m 0755 target/debug/ramwise $(DESTDIR)/usr/bin/ramwise
	mkdir -pv $(DESTDIR)/usr/share
	mkdir -pv $(DESTDIR)/usr/share/licenses/
	mkdir -pv $(DESTDIR)/usr/share/licenses/ramwise/
	install -m 0644 LICENSE "$(DESTDIR)/usr/share/licenses/ramwise/LICENSE"
