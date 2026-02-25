use criterion::{criterion_group, criterion_main, Criterion};
use monoio_pg::Client as MonoioClient;
use tokio_postgres::NoTls;
use std::time::Duration;

const ITERATIONS: u32 = 100;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Postgres Query");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("Small/monoio-pg", |b| {
        let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
            .with_entries(256)
            .enable_timer()
            .build()
            .unwrap();
        let mut client = rt.block_on(MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))).unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT 1").await.unwrap();
                }
            });
        })
    });

    group.bench_function("Small/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (client, connection) = rt.block_on(tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)).unwrap();
        rt.spawn(async move {
            let _ = connection.await;
        });

        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT 1", &[]).await.unwrap();
                }
            });
        })
    });

    group.bench_function("Wide (10 cols)/monoio-pg", |b| {
        let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
            .with_entries(256)
            .enable_timer()
            .build()
            .unwrap();
        let mut client = rt.block_on(MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))).unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT 1, 2, 3, 4, 5, 6, 7, 8, 9, 10").await.unwrap();
                }
            });
        })
    });

    group.bench_function("Wide (10 cols)/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (client, connection) = rt.block_on(tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)).unwrap();
        rt.spawn(async move {
            let _ = connection.await;
        });

        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT 1, 2, 3, 4, 5, 6, 7, 8, 9, 10", &[]).await.unwrap();
                }
            });
        })
    });

    group.bench_function("Large (100 rows)/monoio-pg", |b| {
        let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
            .with_entries(256)
            .enable_timer()
            .build()
            .unwrap();
        let mut client = rt.block_on(MonoioClient::connect("127.0.0.1:5432", "monoio", Some("monoio"), Some("postgres"))).unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT generate_series(1, 100)").await.unwrap();
                }
            });
        })
    });

    group.bench_function("Large (100 rows)/tokio-postgres", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (client, connection) = rt.block_on(tokio_postgres::connect("host=127.0.0.1 user=monoio password=monoio dbname=postgres", NoTls)).unwrap();
        rt.spawn(async move {
            let _ = connection.await;
        });

        b.iter(|| {
            rt.block_on(async {
                for _ in 0..ITERATIONS {
                    let _ = client.query("SELECT generate_series(1, 100)", &[]).await.unwrap();
                }
            });
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
