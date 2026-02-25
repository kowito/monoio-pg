use criterion::{criterion_group, criterion_main, Criterion};
use monoio_pg::Client as MonoioClient;
use tokio_postgres::NoTls;
use std::time::Duration;

async fn bench_monoio(iterations: u32) {
    let mut client = MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))
        .await
        .expect("failed to connect monoio-pg");

    for _ in 0..iterations {
        let _ = client.query("SELECT 1").await.expect("query failed");
    }
}

async fn bench_tokio(iterations: u32) {
    let (client, connection) = tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)
        .await
        .expect("failed to connect tokio-postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    for _ in 0..iterations {
        let _ = client.query("SELECT 1", &[]).await.expect("query failed");
    }
}

async fn bench_monoio_wide(iterations: u32) {
    let mut client = MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))
        .await
        .expect("failed to connect monoio-pg");

    for _ in 0..iterations {
        let _ = client.query("SELECT 1, 2, 3, 4, 5, 6, 7, 8, 9, 10").await.expect("query failed");
    }
}

async fn bench_tokio_wide(iterations: u32) {
    let (client, connection) = tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)
        .await
        .expect("failed to connect tokio-postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    for _ in 0..iterations {
        let _ = client.query("SELECT 1, 2, 3, 4, 5, 6, 7, 8, 9, 10", &[]).await.expect("query failed");
    }
}

async fn bench_monoio_large(iterations: u32) {
    let mut client = MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))
        .await
        .expect("failed to connect monoio-pg");

    for _ in 0..iterations {
        let _ = client.query("SELECT generate_series(1, 100)").await.expect("query failed");
    }
}

async fn bench_tokio_large(iterations: u32) {
    let (client, connection) = tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)
        .await
        .expect("failed to connect tokio-postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    for _ in 0..iterations {
        let _ = client.query("SELECT generate_series(1, 100)", &[]).await.expect("query failed");
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Postgres Query");
    group.measurement_time(Duration::from_secs(10));
    
    let iterations = 100;

    group.bench_function("Small/monoio-pg", |b| {
        b.iter(|| {
            let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();
            rt.block_on(bench_monoio(iterations));
        })
    });

    group.bench_function("Small/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(bench_tokio(iterations));
        })
    });

    group.bench_function("Wide (10 cols)/monoio-pg", |b| {
        b.iter(|| {
            let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();
            rt.block_on(bench_monoio_wide(iterations));
        })
    });

    group.bench_function("Wide (10 cols)/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(bench_tokio_wide(iterations));
        })
    });

    group.bench_function("Large (100 rows)/monoio-pg", |b| {
        b.iter(|| {
            let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();
            rt.block_on(bench_monoio_large(iterations));
        })
    });

    group.bench_function("Large (100 rows)/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(bench_tokio_large(iterations));
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
