#!/bin/sh

#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#

cargo clean
wasm-pack build --release --target web -- --features "input-jscam, output-wasm, small-wasm, test-fail-warning" --no-default-features
mv pkg/nokhwa* nokhwajs/
