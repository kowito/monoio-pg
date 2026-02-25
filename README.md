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
