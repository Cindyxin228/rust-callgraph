cargo clean
cd ../../
cargo clean
cargo install --debug --locked --path . --force
cd rust-url-main/form_urlencoded
cargo callgraph >output.txt