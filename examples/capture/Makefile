
all: build

.PHONY: app build clean release

build:
	@cargo build --features "input-avfoundation"

clean:
	@cargo clean

release:
	@cargo build --features "input-avfoundation" --release

app:
	@cd example-capture && xcodebuild