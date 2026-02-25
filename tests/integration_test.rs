use monoio_pg::{Client, Pool, Result};

#[monoio::test_all]
async fn test_basic_query() -> Result<()> {
    let mut client: Client = match Client::connect("127.0.0.1:5432", "postgres", Some("postgres"), None).await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Postgres not available, skipping integration test");
            return Ok(());
        }
    };

    let rows = client.query("SELECT 1 as id, 'hello' as name").await?;
    assert_eq!(rows.len(), 1);
    
    let id: i32 = rows[0].get(0)?;
    let name: String = rows[0].get(1)?;
    
    assert_eq!(id, 1);
    assert_eq!(name, "hello");
    
    Ok(())
}

#[monoio::test_all]
async fn test_pool() -> Result<()> {
    let pool = Pool::new("127.0.0.1:5432", "postgres", Some("postgres"), None);
    
    let mut client: Client = match pool.get().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Postgres not available, skipping pool test");
            return Ok(());
        }
    };
    
    let rows = client.query("SELECT now()").await?;
    assert_eq!(rows.len(), 1);
    
    pool.put(client);
    
    let _client2 = pool.get().await?;
    
    Ok(())
}
