
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
wasm-pack test --chrome --headless
```
or test base rust code with cargo
```bash
cargo test
```