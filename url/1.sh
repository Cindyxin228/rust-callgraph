cargo clean
cd ../
cargo clean
cargo install --debug --locked --path . --force
cd url
cargo callgraph >output.txt
