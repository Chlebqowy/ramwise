build:
	cargo build --release
clean:
	cargo clean
install:
	mkdir -p $(DESTDIR)/usr/bin
	install -m 0755 target/release/ramwise $(DESTDIR)/usr/bin/ramwise
