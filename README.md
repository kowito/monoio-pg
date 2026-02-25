# monoio-pg

High-performance asynchronous PostgreSQL driver for the `monoio` runtime.

## Features

- **io_uring support**: Leverages `monoio`'s efficient `io_uring` based asynchronous I/O.
- **Thread-per-core**: Optimized for high-throughput, low-latency performance in a thread-per-core architecture.
- **Handshake & Auth**: Supports standard PostgreSQL handshake and authentication (including SCRAM-SHA-256).
- **Extended Query Protocol**: Full support for `parse`, `bind`, and `execute`.

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
| **Small Query** (`SELECT 1`) | **3.30 ms** | 7.20 ms | **2.18x Faster** |
| **Wide Row** (10 columns) | **3.80 ms** | 7.92 ms | **2.08x Faster** |
| **Large Result** (100 rows) | **4.93 ms** | 8.70 ms | **1.76x Faster** |

### Visual Comparison (Lower is Better)

```text
Postgres Query Performance (100 iterations)
-------------------------------------------

Small Query (SELECT 1)
monoio-pg      [#############] 3.30ms
tokio-postgres [############################] 7.20ms

Wide Row (10 columns)
monoio-pg      [###############] 3.80ms
tokio-postgres [###############################] 7.92ms

Large Result (100 rows)
monoio-pg      [####################] 4.93ms
tokio-postgres [###################################] 8.70ms
```

> Benchmarks performed on a thread-per-core configuration comparing `monoio` with the `Fusion` driver vs `tokio`.

## Roadmap

The goal of `monoio-pg` is to become the fastest, most reliable PostgreSQL driver for the Rust ecosystem.

- [ ] **TLS Support**: Integration with `native-tls` and `rustls`.
- [ ] **Transaction Management**: Support for nested transactions and savepoints.
- [ ] **Statement Caching**: Automatic server-side prepared statement management.
- [ ] **Copy Protocol**: High-performance data ingestion with `COPY`.
- [ ] **Notifications**: Support for `LISTEN` and `NOTIFY`.
- [ ] **Portal Support**: Partial result fetching and cursors.
- [ ] **Complex Types**: Native support for JSONB, Arrays, and Range types.
- [ ] **Benchmarks**: Comprehensive performance comparisons against `tokio-postgres`.

## License

WTFPL
