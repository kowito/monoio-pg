use monoio_pg::{Client, Pool};

const HOST: &str = "127.0.0.1:5432";
const USER: &str = "monoio";
const PASS: &str = "monoio";

async fn get_client() -> Client {
    Client::connect(HOST, USER, Some(PASS), Some("postgres"))
        .await
        .expect("Failed to connect to Postgres")
}

#[monoio::test_all]
async fn test_connect_success() {
    let _client = get_client().await;
}

#[monoio::test_all]
async fn test_connect_wrong_password() {
    let result = Client::connect(HOST, USER, Some("wrongpassword"), None).await;
    assert!(result.is_err());
}

#[monoio::test_all]
async fn test_execute_create_drop() {
    let mut client = get_client().await;

    // Drop if exists
    let _ = client.execute("DROP TABLE IF EXISTS test_execute").await;

    // Create
    client
        .execute("CREATE TABLE test_execute (id INT)")
        .await
        .unwrap();

    // Insert
    client
        .execute("INSERT INTO test_execute VALUES (1)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO test_execute VALUES (2)")
        .await
        .unwrap();

    // Drop
    client.execute("DROP TABLE test_execute").await.unwrap();
}

#[monoio::test_all]
async fn test_query_simple() {
    let mut client = get_client().await;
    let rows = client
        .query("SELECT 42 as num, 'hello' as str")
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);

    let num: i32 = rows[0].get(0).unwrap();
    let str_val: String = rows[0].get(1).unwrap();

    assert_eq!(num, 42);
    assert_eq!(str_val, "hello");
}

#[monoio::test_all]
async fn test_query_multiple_rows() {
    let mut client = get_client().await;
    let _ = client.execute("DROP TABLE IF EXISTS test_multi").await;
    client
        .execute("CREATE TABLE test_multi (id INT)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO test_multi VALUES (1), (2), (3)")
        .await
        .unwrap();

    let rows = client
        .query("SELECT id FROM test_multi ORDER BY id")
        .await
        .unwrap();
    assert_eq!(rows.len(), 3);

    let id1: i32 = rows[0].get(0).unwrap();
    let id2: i32 = rows[1].get(0).unwrap();
    let id3: i32 = rows[2].get(0).unwrap();

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    client.execute("DROP TABLE test_multi").await.unwrap();
}

#[monoio::test_all]
async fn test_query_null_values() {
    let mut client = get_client().await;
    let _ = client.execute("DROP TABLE IF EXISTS test_null").await;
    client
        .execute("CREATE TABLE test_null (id INT, val TEXT)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO test_null VALUES (1, NULL)")
        .await
        .unwrap();

    let rows = client.query("SELECT val FROM test_null").await.unwrap();
    assert_eq!(rows.len(), 1);

    // In our simplified FromSql, we might error on NULL if we try to get a String instead of Option<String>.
    // Let's test if getting an Option works. Wait, we haven't implemented FromSql for Option<T>.
    // Let's use get_raw.
    let raw = rows[0].get_raw(0);
    assert!(raw.is_none());

    client.execute("DROP TABLE test_null").await.unwrap();
}

#[monoio::test_all]
async fn test_query_syntax_error() {
    let mut client = get_client().await;
    let result = client.query("SELECT * FROM nonexistent_table").await;
    assert!(result.is_err());

    // Connection should still be usable after an error in simple query protocol!
    // (Postgres automatically goes back to ReadyForQuery).
    let rows = client.query("SELECT 1").await.unwrap();
    assert_eq!(rows.len(), 1);
}

#[monoio::test_all]
async fn test_pool() {
    let pool = Pool::new(HOST, USER, Some(PASS), Some("postgres"));

    let mut client1 = pool.get().await.unwrap();
    client1.execute("SELECT 1").await.unwrap();

    pool.put(client1);

    // Should reuse the connection
    let mut client2 = pool.get().await.unwrap();
    client2.execute("SELECT 2").await.unwrap();
}

#[monoio::test_all]
async fn test_data_types() {
    let mut client = get_client().await;
    // Postgres boolean is 't' or 'f' but wait, query returns text format integers and bools.
    // 't' is length 1 in text format. Our string decoding should handle "t".
    let rows = client
        .query("SELECT true::boolean, false::boolean")
        .await
        .unwrap();

    // For binary format, boolean is bool
    let t_val: bool = rows[0].get(0).unwrap();
    let f_val: bool = rows[0].get(1).unwrap();

    assert_eq!(t_val, true);
    assert_eq!(f_val, false);
}
