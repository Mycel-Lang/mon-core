use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mon_core::{analyze, lexer::Lexer, parser::Parser};

// ============================================================================
// Test Data: Varying Complexity and Size
// ============================================================================

const TINY_MON: &str = r#"{ value: 42 }"#;

const SMALL_MON: &str = r#"{
    name: "test",
    version: 1.0,
    enabled: true,
    tags: ["a", "b", "c"]
}"#;

const MEDIUM_MON: &str = r#"{
    Config: #struct {
        host(String),
        port(Number),
        ssl(Boolean) = false,
        retries(Number) = 3
    },
    
    Status: #enum { Active, Inactive, Pending },
    
    &defaults: {
        ssl: true,
        retries: 5,
        timeout: 30
    },
    
    servers: [
        { ...*defaults, host: "server1.com", port: 8080, status: $Status.Active },
        { ...*defaults, host: "server2.com", port: 8081, status: $Status.Active },
        { ...*defaults, host: "server3.com", port: 8082, status: $Status.Inactive }
    ],
    
    production :: Config = {
        host: "prod.example.com",
        port: 443,
        ssl: true
    }
}"#;

const LARGE_MON: &str = r#"{
    User: #struct {
        id(Number),
        name(String),
        email(String),
        roles([String...]) = [],
        metadata(Any) = null
    },
    
    Permission: #enum { Read, Write, Execute, Admin },
    
    Resource: #struct {
        path(String),
        permissions([Permission...])
    },
    
    &admin_user: {
        id: 1,
        name: "Admin",
        email: "admin@example.com",
        roles: ["admin", "superuser"]
    },
    
    users: [
        *admin_user,
        { id: 2, name: "Alice", email: "alice@example.com", roles: ["developer", "reviewer"] },
        { id: 3, name: "Bob", email: "bob@example.com", roles: ["developer"] },
        { id: 4, name: "Charlie", email: "charlie@example.com", roles: ["viewer"] },
        { id: 5, name: "David", email: "david@example.com", roles: ["developer", "ops"] }
    ],
    
    resources: [
        { path: "/api/users", permissions: [$Permission.Read, $Permission.Write] },
        { path: "/api/admin", permissions: [$Permission.Admin] },
        { path: "/api/metrics", permissions: [$Permission.Read] },
        { path: "/api/config", permissions: [$Permission.Read, $Permission.Write, $Permission.Admin] }
    ],
    
    system_config: {
        api_version: "2.0",
        debug: false,
        max_connections: 1000,
        timeout_seconds: 30,
        cache: {
            enabled: true,
            ttl: 3600,
            max_size: 10485760
        },
        logging: {
            level: "info",
            format: "json",
            output: "stdout"
        }
    }
}"#;

// Generate very large MON for stress testing
fn generate_xlarge_mon(array_size: usize) -> String {
    let mut mon = String::from("{\n    items: [\n");
    for i in 0..array_size {
        mon.push_str(&format!(
            "        {{ id: {}, name: \"Item {}\", value: {}, active: {} }},\n",
            i,
            i,
            i * 100,
            i % 2 == 0
        ));
    }
    mon.push_str("    ]\n}");
    mon
}

// ============================================================================
// Lexer Benchmarks
// ============================================================================

fn bench_lexer_tiny(c: &mut Criterion) {
    c.bench_function("lexer_tiny", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(TINY_MON));
            lexer.lex()
        })
    });
}

fn bench_lexer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_by_size");

    for (name, source) in [
        ("tiny", TINY_MON),
        ("small", SMALL_MON),
        ("medium", MEDIUM_MON),
        ("large", LARGE_MON),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, src| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(src));
                lexer.lex()
            })
        });
    }

    group.finish();
}

fn bench_lexer_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_array_scaling");

    for size in [10, 50, 100, 500, 1000] {
        let source = generate_xlarge_mon(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &source, |b, src| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(src));
                lexer.lex()
            })
        });
    }

    group.finish();
}

// ============================================================================
// Parser Benchmarks
// ============================================================================

fn bench_parser_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_by_size");

    for (name, source) in [
        ("tiny", TINY_MON),
        ("small", SMALL_MON),
        ("medium", MEDIUM_MON),
        ("large", LARGE_MON),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, src| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(src)).unwrap();
                parser.parse_document()
            })
        });
    }

    group.finish();
}

fn bench_parser_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_array_scaling");

    for size in [10, 50, 100, 500, 1000] {
        let source = generate_xlarge_mon(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &source, |b, src| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(src)).unwrap();
                parser.parse_document()
            })
        });
    }

    group.finish();
}

// ============================================================================
// End-to-End Analysis Benchmarks
// ============================================================================

fn bench_e2e_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_analysis");

    for (name, source) in [
        ("tiny", TINY_MON),
        ("small", SMALL_MON),
        ("medium", MEDIUM_MON),
        ("large", LARGE_MON),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, src| {
            b.iter(|| analyze(black_box(src), "benchmark.mon"))
        });
    }

    group.finish();
}

fn bench_e2e_with_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_with_json_serialization");

    for (name, source) in [
        ("tiny", TINY_MON),
        ("small", SMALL_MON),
        ("medium", MEDIUM_MON),
        ("large", LARGE_MON),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, src| {
            b.iter(|| {
                let result = analyze(black_box(src), "benchmark.mon").unwrap();
                result.to_json()
            })
        });
    }

    group.finish();
}

fn bench_e2e_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_array_scaling");

    for size in [10, 50, 100, 500, 1000] {
        let source = generate_xlarge_mon(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &source, |b, src| {
            b.iter(|| analyze(black_box(src), "benchmark.mon"))
        });
    }

    group.finish();
}

// ============================================================================
// Real-World Scenario Benchmarks
// ============================================================================

fn bench_realistic_config(c: &mut Criterion) {
    // Simulates a realistic application configuration file
    let config = r#"{
        Database: #struct {
            host(String),
            port(Number),
            pool_size(Number) = 10
        },
        
        LogLevel: #enum { Debug, Info, Warn, Error },
        
        database :: Database = {
            host: "localhost",
            port: 5432,
            pool_size: 20
        },
        
        cache: {
            enabled: true,
            ttl_seconds: 3600,
            max_entries: 10000
        },
        
        logging: {
            level: $LogLevel.Info,
            format: "json"
        },
        
        features: {
            auth_enabled: true,
            rate_limiting: true,
            compression: false
        }
    }"#;

    c.bench_function("realistic_app_config", |b| {
        b.iter(|| analyze(black_box(config), "app_config.mon"))
    });
}

fn bench_complex_schema(c: &mut Criterion) {
    // Simulates complex type definitions with validation
    let schema = r#"{
        Address: #struct {
            street(String),
            city(String),
            zip(String),
            country(String) = "USA"
        },
        
        ContactType: #enum { Email, Phone, Slack, Teams },
        
        Contact: #struct {
            type(ContactType),
            value(String),
            primary(Boolean) = false
        },
        
        Person: #struct {
            name(String),
            addresses([Address...]),
            contacts([Contact...])
        },
        
        person :: Person = {
            name: "John Doe",
            addresses: [
                { street: "123 Main St", city: "Boston", zip: "02101" }
            ],
            contacts: [
                { type: $ContactType.Email, value: "john@example.com", primary: true },
                { type: $ContactType.Phone, value: "+1-555-0100" }
            ]
        }
    }"#;

    c.bench_function("complex_schema_validation", |b| {
        b.iter(|| analyze(black_box(schema), "schema.mon"))
    });
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    lexer_benches,
    bench_lexer_tiny,
    bench_lexer_sizes,
    bench_lexer_scaling
);

criterion_group!(parser_benches, bench_parser_sizes, bench_parser_scaling);

criterion_group!(
    e2e_benches,
    bench_e2e_analysis,
    bench_e2e_with_serialization,
    bench_e2e_scaling
);

criterion_group!(
    realistic_benches,
    bench_realistic_config,
    bench_complex_schema
);

criterion_main!(
    lexer_benches,
    parser_benches,
    e2e_benches,
    realistic_benches
);
