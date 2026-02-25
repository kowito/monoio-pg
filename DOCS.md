# monoio-pg Manual

`monoio-pg` is a high-performance PostgreSQL driver designed specifically for the `monoio` asynchronous runtime. It leverages `io_uring` and a thread-per-core architecture to provide superior throughput and low latency.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Getting Started](#getting-started)
- [Connection Management](#connection-management)
- [Executing Queries](#executing-queries)
- [Working with Rows](#working-with-rows)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Core Concepts

- **Thread-per-core**: `monoio-pg` is designed to run on `monoio`. Each thread manages its own events and connections, minimizing lock contention.
- **io_uring**: High-efficiency I/O subsystem on Linux, providing asynchronous system calls.
- **Extended Query Protocol**: `monoio-pg` uses the PostgreSQL extended query protocol, which is more robust and efficient than simple queries.

## Getting Started

Add `monoio-pg` and `monoio` to your `Cargo.toml`:

```toml
[dependencies]
monoio = { version = "0.2", features = ["macros"] }
monoio-pg = "0.1"
```

## Connection Management

### Direct Connection

You can connect directly to a PostgreSQL server using the `Client` struct.

```rust
use monoio_pg::Client;

let client = Client::connect(
    "127.0.0.1:5432",
    "postgres",
    Some("password"),
    Some("database_name")
).await?;
```

### Connection Pooling (Thread-local)

Since `monoio` is thread-per-core, the pool provided by `monoio-pg` is thread-local. This avoids sharing clients across threads, which is a key performance optimization.

```rust
use monoio_pg::Pool;

let pool = Pool::new(
    "127.0.0.1:5432",
    "postgres",
    Some("password"),
    Some("database_name")
);

// Get a client from the pool
let mut client = pool.get().await?;

// Use the client...

// Return the client to the pool
pool.put(client);
```

## Executing Queries

### `execute`

Use `execute` for queries that don't return rows (e.g., `INSERT`, `UPDATE`, `CREATE TABLE`).

```rust
client.execute("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT)").await?;
```

### `query`

Use `query` for queries that return rows (e.g., `SELECT`).

```rust
let rows = client.query("SELECT id, name FROM users").await?;
```

## Working with Rows

`monoio-pg` provides a type-safe way to extract data from rows using the `get` method, which supports types implementing the `FromSql` trait from the `postgres-types` crate.

```rust
for row in rows {
    let id: i32 = row.get(0)?;
    let name: String = row.get(1)?;
    println!("User: {} (ID: {})", name, id);
}
```

Common supported types:
- `i32`, `i64`
- `String`, `&str`
- `bool`
- `Vec<u8>`
- `Option<T>` (for nullable columns)

## Error Handling

All database operations return a `Result<T, monoio_pg::Error>`.

```rust
use monoio_pg::Error;

match client.query("SELECT * FROM non_existent").await {
    Ok(_) => println!("Success"),
    Err(Error::Protocol(msg)) => eprintln!("Postgres Error: {}", msg),
    Err(e) => eprintln!("Other Error: {}", e),
}
```

## Examples

For more comprehensive examples, check the `examples/` directory in the repository.

### Basic SELECT

```rust
#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("127.0.0.1:5432", "postgres", None, None).await?;
    
    let rows = client.query("SELECT 'Hello Monoio'").await?;
    let greeting: String = rows[0].get(0)?;
    
    println!("{}", greeting);
    Ok(())
}
```
