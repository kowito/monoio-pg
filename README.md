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

## License

MIT OR Apache-2.0
