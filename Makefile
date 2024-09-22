# Detect OS
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Linux)
    TARGET := x86_64-unknown-linux-gnu
    BINARY := rustylines
endif

ifeq ($(UNAME_S),Darwin)
    TARGET := x86_64-apple-darwin
    BINARY := rustylines
endif

ifeq ($(findstring MINGW,$(UNAME_S)),MINGW)
    TARGET := x86_64-pc-windows-gnu
    BINARY := rustylines.exe
endif

# Build target
build:
	cargo build --release --target $(TARGET)

# Copy binary to root
copy:
	cp target/$(TARGET)/release/$(BINARY) .

# Combined build and copy
build-and-copy: build copy

# Clean copied binary
clean-binary:
	rm -f $(BINARY)

.PHONY: build copy build-and-copy clean-binary
