
all: build

.PHONY: app build clean release

build:
	@cargo build

clean:
	@cargo clean

release:
	@cargo build --release

app:
	@cd example-capture && xcodebuild