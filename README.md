Reis MMIO is a toolkit based on svd2rust that can ingest SVDs files and generate C++ abstractions to write and read mmios in a very efficiente way.

## Usage
To generate the C++  MMIO abstractions for Ibex demo system run:

```sh
wget https://raw.githubusercontent.com/lowRISC/ibex-demo-system/refs/heads/main/data/ibex.svd -O /tmp/ibex.svd
cargo run inport-svd --svd /tmp/ibex.svd export-cpp --dir /tmp/ --periph-dir /tmp/
```
