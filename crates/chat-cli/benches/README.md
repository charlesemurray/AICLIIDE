# Performance Benchmarks

## Overview

This directory contains performance benchmarks for the Skills & Workflows ToolSpec integration.

## Benchmarks

### toolspec_conversion.rs

Measures the performance of converting skills and workflows to ToolSpec format.

**Benchmarks included**:
1. `skill_to_toolspec` - Simple skill conversion
2. `workflow_to_toolspec` - Simple workflow conversion
3. `skill_to_toolspec_complex` - Complex skill with 10 parameters
4. `workflow_to_toolspec_complex` - Complex workflow with 10 steps

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench toolspec_conversion

# Run with specific filter
cargo bench skill_to_toolspec
```

## Performance Targets

Based on the implementation plan:

- **Skill to ToolSpec conversion**: < 1ms
- **Workflow to ToolSpec conversion**: < 1ms
- **Simple workflow execution**: < 100ms

## Expected Results

### Simple Conversions
- Skill to ToolSpec: ~100-500 microseconds
- Workflow to ToolSpec: ~200-800 microseconds

### Complex Conversions
- Complex skill (10 parameters): ~500-1000 microseconds
- Complex workflow (10 steps): ~1-2 milliseconds

## Notes

- Benchmarks use Criterion for statistical analysis
- Results may vary based on hardware
- Benchmarks run in release mode for accurate measurements
- Each benchmark runs multiple iterations for statistical significance

## Interpreting Results

Criterion provides:
- **Mean**: Average execution time
- **Std Dev**: Standard deviation
- **Median**: Middle value of all measurements
- **MAD**: Median Absolute Deviation

Look for:
- Consistent performance across runs
- Low standard deviation
- No performance regressions between versions

## Adding New Benchmarks

To add a new benchmark:

1. Create a benchmark function:
```rust
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| black_box(my_function()));
    });
}
```

2. Add to criterion_group:
```rust
criterion_group!(benches, bench_my_feature);
```

3. Run and verify:
```bash
cargo bench --bench toolspec_conversion
```
