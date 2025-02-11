cargo clean
cd ../
cargo clean
cargo install --debug --locked --path . --force
cd example_code
cargo callgraph 
