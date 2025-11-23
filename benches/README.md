# MON-Core Performance Benchmarks

## Overview

Professional-grade benchmark suite following industry best practices (Chandler Carruth / Oracle methodology).

## Benchmark Categories

### 1. Lexer Benchmarks

- `lexer_tiny` - Single baseline measurement
- `lexer_by_size` - Performance across input sizes (tiny → large)
- `lexer_array_scaling` - Scalability with array elements (10 → 1000)

### 2. Parser Benchmarks

- `parser_by_size` - Parsing performance by complexity
- `parser_array_scaling` - Parser scalability test

### 3. End-to-End Benchmarks

- `e2e_analysis` - Full pipeline (lex → parse → resolve)
- `e2e_with_json_serialization` - Including JSON output
- `e2e_array_scaling` - E2E scalability

### 4. Real-World Scenarios

- `realistic_app_config` - Typical application configuration
- `complex_schema_validation` - Complex type validation

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific category
cargo bench --bench mon_benchmarks -- lexer

# Run with baseline for comparison
cargo bench --bench mon_benchmarks -- --save-baseline main

# Compare against baseline
cargo bench --bench mon_benchmarks -- --baseline main

# Generate flamegraphs (requires cargo-flamegraph)
cargo flamegraph --bench mon_benchmarks
```

## Input Sizes

| Size   | Lines   | Bytes   | Description                   |
| ------ | ------- | ------- | ----------------------------- |
| Tiny   | 1       | 14      | Simple value                  |
| Small  | 6       | 120     | Basic object                  |
| Medium | 20      | 600+    | With types & spreads          |
| Large  | 60+     | 2000+   | Complex schema                |
| XLarge | Dynamic | Dynamic | Scaling tests (10-1000 items) |

## Metrics Tracked

- **Throughput**: Bytes/second or Elements/second
- **Latency**: Mean, median, std dev
- **Scaling**: O(n) characteristics
- **Statistical Rigor**: Confidence intervals via Criterion

## Results Location

- HTML Reports: `target/criterion/`
- CSV Data: `target/criterion/*/base/estimates.json`
- Comparison Plots: `target/criterion/*/report/index.html`

## Performance Targets

Based on typical use cases:

- Lexer: >10 MB/s
- Parser: >5 MB/s
- E2E Analysis: >2 MB/s
- Serialization: >10 MB/s

## Interpreting Results

Criterion provides:

- **Mean**: Average performance
- **Std Dev**: Consistency
- **Outliers**: Reliability indicators
- **R²**: Linear regression fit (for scaling)

Look for:

- Linear scaling O(n) for large inputs
- Consistent performance across runs
- Minimal outliers

## Adding New Benchmarks

```rust
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function())
        })
    });
}

criterion_group!(my_group, bench_my_feature);
criterion_main!(my_group);
```

## CI Integration

```yaml
# Add to GitHub Actions
- name: Run Benchmarks
  run: cargo bench --bench mon_benchmarks -- --output-format bencher | tee output.txt
```
