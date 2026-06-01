CARGO = $(HOME)/.cargo/bin/cargo
FLUTTER = $(HOME)/flutter/bin/flutter

.PHONY: test build check render run clean

## Rust DSP (Phase 1)
test:
	cd rust && $(CARGO) test

check:
	cd rust && $(CARGO) check

build:
	cd rust && $(CARGO) build --release

render:
	cd rust && $(CARGO) test render_all -- --nocapture

## Flutter app (Phase 2+)
run:
	$(FLUTTER) run -d linux

flutter-build:
	$(FLUTTER) build linux

codegen:
	flutter_rust_bridge_codegen generate

clean:
	rm -rf build/
	cd rust && $(CARGO) clean
