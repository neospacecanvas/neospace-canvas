
### build and test
This lib is written so as much code as possible can be built and tested with both cargo and wasm-pack.
Some things like webworkers can only run with wasm-pack as they exist only in browser implementations.

build project wasm binaries
```bash
wasm-pack build --target web
```
or build base rust with cargo
```bash
cargo build
```

testing
```bash
wasm-pack test --node
```
or test base rust code with cargo
```bash
cargo test
```
running with debug logging
```bash
cargo test -- --nocapture
```
testing with warnings turned off (this flag does force recompile)
```bash
RUSTFLAGS=-Awarnings cargo test
```
testing with warnings turned off and debug logging (the best way)
```bash
RUSTFLAGS=-Awarnings cargo test
```
