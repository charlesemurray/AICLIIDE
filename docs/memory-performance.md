# Memory System Performance Testing

## Performance Targets

- **Store Operation**: < 50ms per interaction
- **Recall Operation**: < 100ms per query
- **List Operation**: < 50ms per request
- **Session-Filtered Recall**: < 100ms per query

## Running Benchmarks

```bash
cargo run --release --bin memory_benchmark -p cortex-memory
```

## Test Scenarios

### 1. Store Performance
- Store 100 interactions
- Measure average time per store
- Verify < 50ms target

### 2. Recall Performance
- Perform 50 recall queries
- Measure average time per recall
- Verify < 100ms target

### 3. Session-Filtered Recall
- Perform 50 session-specific recalls
- Measure average time per query
- Verify < 100ms target

### 4. List Performance
- Perform 100 list operations
- Measure average time per list
- Verify < 50ms target

## Large Dataset Testing

Test with 10,000+ memories:

```bash
# Create test dataset
cargo test -p cortex-memory --test large_dataset -- --ignored

# Run performance tests
cargo run --release --bin memory_benchmark -p cortex-memory
```

## Optimization Notes

### Current Optimizations
- SQLite indexes on session_id and timestamp
- Embedding caching in database
- Efficient HNSW vector search
- Connection pooling for concurrent access

### Future Optimizations
- Batch insert operations
- Lazy embedding generation
- Memory-mapped database access
- Parallel query execution

## Platform Testing

Test on all supported platforms:

### macOS (x86_64)
```bash
cargo run --release --bin memory_benchmark -p cortex-memory
```

### macOS (ARM64)
```bash
cargo run --release --bin memory_benchmark -p cortex-memory
```

### Linux (x86_64)
```bash
cargo run --release --bin memory_benchmark -p cortex-memory
```

### Linux (ARM64)
```bash
cargo run --release --bin memory_benchmark -p cortex-memory
```

## Results

### Expected Performance (Release Build)

| Operation | Target | Typical |
|-----------|--------|---------|
| Store | < 50ms | ~10-20ms |
| Recall | < 100ms | ~30-60ms |
| Session Recall | < 100ms | ~30-60ms |
| List | < 50ms | ~5-15ms |

### Scaling Characteristics

- **100 memories**: All operations within targets
- **1,000 memories**: Slight increase in recall time (~10-20%)
- **10,000 memories**: Recall time may approach 100ms limit
- **100,000 memories**: May require optimization or cleanup

## Monitoring

In production, monitor:
- Average recall latency
- 95th percentile recall latency
- Database size growth
- Memory usage

## Troubleshooting

**Slow recall times?**
- Check database size with `/memory stats`
- Run `/memory cleanup --force` to remove old data
- Verify SQLite indexes are present
- Check disk I/O performance

**High memory usage?**
- Reduce `memory.maxSizeMb` setting
- Enable automatic cleanup
- Reduce retention period

**Database locked errors?**
- Reduce concurrent access
- Increase SQLite timeout
- Check for long-running queries
