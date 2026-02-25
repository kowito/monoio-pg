use monoio_pg::pool::Pool;

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new("127.0.0.1:5432", "postgres", Some("password"), Some("postgres"));

    // 1. Get a client from the pool
    let mut client = pool.get().await?;

    // 2. Execute a simple query
    client.execute("CREATE TABLE IF NOT EXISTS test (id INT, name TEXT)").await?;
    client.execute("INSERT INTO test (id, name) VALUES (1, 'monoio')").await?;

    // 3. Run a query and get rows
    let rows = client.query("SELECT id, name FROM test").await?;
    for row in rows {
        let id_raw = row.get_raw(0);
        let name_raw = row.get_raw(1);
        println!("Row: id={:?}, name={:?}", id_raw, name_raw);
    }

    // 4. Put the client back in the pool
    pool.put(client);

    Ok(())
}
