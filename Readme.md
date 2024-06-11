To run test coverage

Install (grcov)[https://github.com/mozilla/grcov]

run
`CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test`

then
`grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html`

then open the index.html at target/coverage/

on wasm_bindings run `wasm-pack build --target nodejs`

on gridlab-ts run `yarn add ../crates/wasm_bindings/pkg` then `yarn start:dev`