# OpenFusionMap-rs

OpenFusion player map in your browser.

OpenFusionMap-rs is a rewrite of [OpenFusionMap](https://github.com/OpenFusionProject/OpenFusionMap) in Rust using [axum](https://github.com/tokio-rs/axum) and [ffmonitor](https://github.com/OpenFusionProject/ffmonitor)

## Building
```
cargo build --release
```

## Usage
```
Usage: openfusionmap [OPTIONS]

Options:
  -b, --bind-addr <BIND_ADDR>        The address to bind the HTTP server to [default: 127.0.0.1:8080]
  -m, --monitor-addr <MONITOR_ADDR>  The address of the OpenFusion monitor to connect to [default: 127.0.0.1:8003]
  -h, --help                         Print help
```
Then connect in your browser.