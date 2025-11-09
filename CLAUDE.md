# CLAUDE.md - Development Documentation

## Project Overview

**CSV-SQL Streaming Loader** - A high-performance Rust CLI tool for streaming CSV data into PostgreSQL databases.

### Key Features
- **Type Inference**: Automatically detect column types from CSV data
- **Streaming Architecture**: Handle multi-GB files without loading into memory
- **Batched COPY**: Use PostgreSQL's COPY protocol for optimal bulk loading
- **Progress Tracking**: Real-time UI showing progress, throughput, and ETA
- **Retry Logic**: Resilient error handling with configurable retry strategies
- **Metrics**: Emit detailed performance and operational metrics
- **Testing**: Comprehensive unit and property-based tests

## Architecture

### Phase 1: Parser + Schema Inference
**Status**: 🔴 Not Started

#### Components
1. **CSV Parser**
   - Stream-based parsing using `csv` crate
   - Memory-efficient row iteration
   - Handle various delimiters and quote styles
   - Support for headers/no-headers mode

2. **Schema Inference Engine**
   - Sample-based type detection (configurable sample size)
   - Type hierarchy: NULL → INT → FLOAT → TIMESTAMP → TEXT
   - Confidence scoring for inferred types
   - Handle edge cases (empty strings, nulls, mixed types)
   - Generate CREATE TABLE statements

#### Files
- `src/parser.rs` - CSV streaming parser
- `src/schema.rs` - Type inference and schema generation
- `src/types.rs` - Type system definitions

### Phase 2: PostgreSQL COPY Protocol
**Status**: 🔴 Not Started

#### Components
1. **Connection Pool**
   - Use `tokio-postgres` for async I/O
   - Configurable pool size
   - Connection retry logic
   - SSL/TLS support

2. **Batched COPY Handler**
   - Batch rows (default: 10,000 rows per batch)
   - Binary COPY format for performance
   - Transaction management
   - Error isolation per batch

#### Files
- `src/db/mod.rs` - Database module
- `src/db/connection.rs` - Connection pool management
- `src/db/copy.rs` - COPY protocol implementation
- `src/db/batch.rs` - Batching logic

### Phase 3: Progress UI + Retries
**Status**: 🔴 Not Started

#### Components
1. **Progress Display**
   - Use `indicatif` for progress bars
   - Show: rows processed, throughput (rows/sec), ETA, current batch
   - Multi-bar for concurrent operations
   - Graceful terminal handling

2. **Retry Strategy**
   - Exponential backoff with jitter
   - Configurable max retries
   - Dead letter queue for failed batches
   - Detailed error logging

#### Files
- `src/progress.rs` - Progress tracking and display
- `src/retry.rs` - Retry logic and strategies
- `src/errors.rs` - Error types and handling

### Phase 4: Tests + Performance
**Status**: 🔴 Not Started

#### Components
1. **Unit Tests**
   - Parser correctness
   - Schema inference accuracy
   - Batch handling
   - Error scenarios

2. **Property Tests**
   - Use `proptest` or `quickcheck`
   - Generate random CSVs
   - Verify data integrity end-to-end
   - Stress test batch boundaries

3. **Performance Benchmarks**
   - Use `criterion` for benchmarks
   - Measure throughput (MB/s, rows/s)
   - Memory profiling
   - Compare with traditional `psql COPY`

#### Files
- `tests/unit/` - Unit tests
- `tests/property/` - Property-based tests
- `benches/` - Criterion benchmarks
- `docs/PERFORMANCE.md` - Performance notes and tuning

## CLI Interface

### Command Structure
```bash
csv-sql-loader [OPTIONS] <CSV_FILE> <CONNECTION_STRING>

Options:
  -t, --table <TABLE>          Target table name (default: inferred from filename)
  -b, --batch-size <SIZE>      Rows per batch (default: 10000)
  -s, --sample-size <SIZE>     Rows to sample for type inference (default: 1000)
  --create-table               Create table if it doesn't exist
  --drop-table                 Drop table before loading
  --delimiter <CHAR>           CSV delimiter (default: ,)
  --no-header                  CSV has no header row
  --max-retries <NUM>          Maximum retry attempts (default: 3)
  --dry-run                    Show inferred schema without loading
  -v, --verbose                Verbose output
  -q, --quiet                  Suppress progress display
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples
```bash
# Basic usage
csv-sql-loader data.csv "postgresql://localhost/mydb"

# Custom table with schema preview
csv-sql-loader --table users --dry-run users.csv "postgresql://localhost/mydb"

# Drop existing table and load with custom batch size
csv-sql-loader --drop-table --batch-size 50000 large.csv "postgresql://localhost/mydb"
```

## Technology Stack

### Core Dependencies
- **clap** (v4): CLI argument parsing
- **csv**: Streaming CSV parser
- **tokio**: Async runtime
- **tokio-postgres**: PostgreSQL async driver
- **indicatif**: Progress bars
- **serde**: Serialization/deserialization
- **anyhow**: Error handling
- **tracing**: Structured logging

### Development Dependencies
- **proptest**: Property-based testing
- **criterion**: Benchmarking
- **tempfile**: Test fixtures
- **mockall**: Mocking

## Development Workflow

### Branch Strategy
- **Development Branch**: `claude/initial-setup-claude-md-011CUxmRM4D87Y4YjSZahd3z`
- **Target Branch**: `main` (merge when complete)

### Build Commands
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check linting
cargo clippy

# Format code
cargo fmt
```

### Release Process
1. Build release binaries: `cargo build --release`
2. Run full test suite: `cargo test --all-features`
3. Generate benchmarks: `cargo bench`
4. Update CHANGELOG.md
5. Tag release: `git tag v0.1.0`
6. Create GitHub release with binaries

## Implementation Plan

### Sprint 1: Foundation (Phase 1)
- [x] Initialize Rust project with Cargo.toml
- [ ] Implement CSV streaming parser
- [ ] Build type inference engine
- [ ] Add schema generation
- [ ] Unit tests for parser and schema

### Sprint 2: Database Integration (Phase 2)
- [ ] Set up PostgreSQL connection pool
- [ ] Implement COPY protocol handler
- [ ] Add batching logic
- [ ] Error handling and transactions
- [ ] Integration tests with test database

### Sprint 3: UX & Reliability (Phase 3)
- [ ] Build progress UI with indicatif
- [ ] Implement retry logic with backoff
- [ ] Add comprehensive logging
- [ ] CLI argument parsing
- [ ] End-to-end testing

### Sprint 4: Polish & Performance (Phase 4)
- [ ] Property-based tests
- [ ] Performance benchmarks
- [ ] Documentation (README, examples, perf notes)
- [ ] Release binaries for multiple platforms
- [ ] Final polish and bug fixes

## Performance Goals

### Target Metrics
- **Throughput**: > 100K rows/second on commodity hardware
- **Memory**: < 100MB for files of any size (streaming)
- **Latency**: Sub-second startup time
- **Reliability**: 99.9% success rate with retries

### Optimization Strategies
- Binary COPY format (vs text)
- Batch size tuning (trade-off: memory vs network roundtrips)
- Connection pooling
- Zero-copy parsing where possible
- Async I/O throughout

## Testing Strategy

### Coverage Goals
- **Unit Tests**: > 80% line coverage
- **Integration Tests**: All major workflows
- **Property Tests**: Data integrity guarantees
- **Benchmarks**: Regression detection

### Test Data
- Small files (< 1MB): Unit tests
- Medium files (10-100MB): Integration tests
- Large files (> 1GB): Performance tests
- Edge cases: Empty, single row, malformed, mixed types

## Security Considerations

- [ ] SQL injection prevention (parameterized queries only)
- [ ] Connection string validation
- [ ] TLS/SSL support for database connections
- [ ] Input validation (file paths, table names)
- [ ] Resource limits (max file size, max batch size)

## Known Limitations & Future Work

### V1 Scope Limitations
- PostgreSQL only (no MySQL, SQLite, etc.)
- CSV only (no JSON, Parquet, etc.)
- Single table at a time
- No data transformations (use as-is from CSV)

### Future Enhancements
- Multi-database support (MySQL, SQLite, Snowflake)
- Additional file formats (JSON, Parquet, Arrow)
- Column mapping and transformations
- Upsert support (INSERT ... ON CONFLICT)
- Parallel loading with multiple connections
- Cloud storage input (S3, GCS, Azure Blob)

## License

MIT (or Apache 2.0 - TBD)

---

**Last Updated**: 2025-11-09
**Status**: Initial Setup
**Current Phase**: Phase 0 - Project Initialization
