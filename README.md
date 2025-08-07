zermio (ZERo cost Memory mapped Input and Output) is a toolkit based on svd2rust that can ingest SVDs files and generate C++ abstractions to write and read mmios in a very efficiente way.
In tests with gcc 15 and clang 19 with optimization `O2` the abstraction was completed inlined to a few instructions.

## Usage
Generating the C++ MMIO abstractions for Ibex demo system with the command:

```sh
wget https://raw.githubusercontent.com/lowRISC/ibex-demo-system/refs/heads/main/data/ibex.svd -O /tmp/ibex.svd
cargo run import-svd --svd /tmp/ibex.svd export-cpp --dir /tmp/ --periph-dir /tmp/
```

Customizing the license header on each generated source file.

```sh
cargo run import-svd --header-file=/tmp/header.md --svd /tmp/ibex.svd export-cpp --dir /tmp/ --periph-dir /tmp/
```
