cargo clean
cd ../../
cargo clean
cargo install --debug --locked --path . --force
cd rust-url-main/idna
cargo callgraph
