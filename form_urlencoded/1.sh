cargo clean
cd ../
cargo clean
cargo install --debug --locked --path . --force
cd form_urlencoded
cargo callgraph >output.txt
