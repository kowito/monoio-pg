# monoio-pg

[![Crates.io](https://img.shields.io/crates/v/monoio-pg.svg)](https://crates.io/crates/monoio-pg)
[![Downloads](https://img.shields.io/crates/d/monoio-pg.svg)](https://crates.io/crates/monoio-pg)
[![Docs.rs](https://docs.rs/monoio-pg/badge.svg)](https://docs.rs/monoio-pg)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-blue.svg)](https://github.com/kowito/monoio-pg)
[![License](https://img.shields.io/badge/license-WTFPL-blue.svg)](https://github.com/kowito/monoio-pg/blob/main/LICENSE)
[![CI](https://github.com/kowito/monoio-pg/actions/workflows/publish.yml/badge.svg)](https://github.com/kowito/monoio-pg/actions)

High-performance asynchronous PostgreSQL driver for the `monoio` runtime.

## Features

- **io_uring support**: Leverages `monoio`'s efficient `io_uring` based asynchronous I/O.
- **Thread-per-core**: Optimized for high-throughput, low-latency performance in a thread-per-core architecture.
- **Handshake & Auth**: Supports standard PostgreSQL handshake and authentication (including SCRAM-SHA-256).
- **Extended Query Protocol**: Full support for `parse`, `bind`, and `execute`.
- **Zero-Copy Architecture**: Minimized memory allocations and data copying during query processing.
- **Implicit Statement Caching**: Automatically reuses parsed statements and row descriptions for maximum efficiency.
- **Automatic CI/CD**: Fully automated versioning and publishing to crates.io on every push.

## Usage

```rust
use monoio_pg::Client;

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("127.0.0.1:5432", "user", Some("password"), Some("database")).await?;
    let rows = client.query("SELECT 1").await?;
    println!("{:?}", rows);
    Ok(())
}
```

## Performance

`monoio-pg` is built for extreme performance. In our benchmarks (running 100 queries per iteration), it consistently outperforms `tokio-postgres` by **1.7x to 2.2x**.

| Benchmark Scenario | `monoio-pg` (Avg) | `tokio-postgres` (Avg) | Speedup |
| :--- | :--- | :--- | :--- |
| **Small Query** (`SELECT 1`) | **1.97 ms** | 6.57 ms | **3.33x Faster** |
| **Wide Row** (10 columns) | **2.08 ms** | 7.10 ms | **3.41x Faster** |
| **Large Result** (100 rows) | **3.59 ms** | 7.93 ms | **2.20x Faster** |

### Visual Comparison (Higher is Better)

```text
Postgres Query Throughput (Higher is Better)
--------------------------------------------

Small Query (SELECT 1)
  monoio-pg      ████████████████████      50,761 req/s
  tokio-postgres ██████                    15,220 req/s

Wide Row (10 columns)
  monoio-pg      ████████████████████      48,076 req/s
  tokio-postgres ██████                    14,084 req/s

Large Result (100 rows)
  monoio-pg      ███████████               27,855 req/s
  tokio-postgres █████                     12,610 req/s
```

> Benchmarks performed on a thread-per-core configuration comparing `monoio` with the `Fusion` driver vs `tokio`.

## Roadmap

The goal of `monoio-pg` is to become the fastest, most reliable PostgreSQL driver for the Rust ecosystem.

- [x] **Benchmarks**: Comprehensive performance comparisons against `tokio-postgres`.
- [x] **Statement Caching**: Implicit management of prepared statements and row descriptions.
- [ ] **TLS Support**: Integration with `native-tls` and `rustls`.
- [ ] **Transaction Management**: Support for nested transactions and savepoints.
- [ ] **Copy Protocol**: High-performance data ingestion with `COPY`.
- [ ] **Notifications**: Support for `LISTEN` and `NOTIFY`.
- [ ] **Portal Support**: Partial result fetching and cursors.
- [ ] **Complex Types**: Native support for JSONB, Arrays, and Range types.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

WTFPL
