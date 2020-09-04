PWD = $(shell pwd)
CARGO = ~/.cargo/bin/cargo

all: build

muslenv:
	# docker inner
	./tools/muslenv.sh

build:
	$(CARGO) build --bins

build_musl: muslenv
	docker exec muslenv make build

release:
	$(CARGO) build --bins --release

release_musl: muslenv
	docker exec muslenv make release

install:
	$(CARGO) install -f --bins --path ./client --root=/usr/local/
	$(CARGO) install -f --bins --path ./server --root=/usr/local/
	$(CARGO) install -f --bins --path ./proxy --root=/usr/local/
	$(CARGO) install -f --bins --path ./rexec --root=/usr/local/

lint:
	$(CARGO) clippy
	cd core && $(CARGO) clippy --features="testmock"
	cd core && $(CARGO) clippy --no-default-features --features="zfs_snapshot"
	cd core && $(CARGO) clippy --no-default-features --features="testmock, zfs_snapshot"
	cd server && $(CARGO) clippy --features="testmock"
	cd server && $(CARGO) clippy --no-default-features --features="zfs_snapshot"
	cd server && $(CARGO) clippy --no-default-features --features="testmock, zfs_snapshot"
	cd proxy && $(CARGO) clippy --features="testmock"

stop:
	- pkill -9 ppserver

test: stop
	$(CARGO) test -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd server && $(CARGO) test --features="testmock" -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd server && $(CARGO) test --no-default-features --features="testmock, zfs_snapshot" -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd proxy && $(CARGO) test --features="testmock" -- --test-threads=1 --nocapture

test_release: stop
	$(CARGO) test --release -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd server && $(CARGO) test --release --features="testmock" -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd server && $(CARGO) test --release --no-default-features --features="testmock, zfs_snapshot" -- --test-threads=1 --nocapture
	-@ pkill -9 integration
	cd proxy && $(CARGO) test --release --features="testmock" -- --test-threads=1 --nocapture

clean:
	@ $(CARGO) clean
	@ find . -type f -name "Cargo.lock" | xargs rm -f

cleanall: clean
	@ find . -type d -name "target" | xargs rm -rf

fmt:
	@ ./tools/fmt.sh

doc:
	$(CARGO) doc --open -p pp
	$(CARGO) doc --open -p ppproxy
	$(CARGO) doc --open -p ppserver
	$(CARGO) doc --open -p ppcore
	$(CARGO) doc --open -p pprexec
