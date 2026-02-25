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

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Postgres Query (SELECT 1)");
    group.measurement_time(Duration::from_secs(10));
    
    let iterations = 100;

    group.bench_function("monoio-pg", |b| {
        b.iter(|| {
            let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();
            rt.block_on(bench_monoio(iterations));
        })
    });

    group.bench_function("tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(bench_tokio(iterations));
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
