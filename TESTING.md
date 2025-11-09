# Testing Guide for CSV-SQL Streaming Loader

## Prerequisites

### 1. PostgreSQL Setup

You need a running PostgreSQL database. Choose one method:

#### Option A: Docker (Easiest)
```bash
# Start PostgreSQL in Docker
docker run --name test-postgres \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  -d postgres:15

# Connection string:
# postgresql://testuser:testpass@localhost:5432/testdb
```

#### Option B: Local PostgreSQL
```bash
# Install PostgreSQL (Ubuntu/Debian)
sudo apt-get install postgresql postgresql-contrib

# Create test database
sudo -u postgres createdb testdb

# Connection string:
# postgresql://postgres@localhost/testdb
```

## Test Scenarios

### Test 1: Dry Run (Schema Preview)
```bash
./target/release/csv-sql-loader \
  --dry-run \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Shows inferred schema
- Displays column types with confidence scores
- Shows CREATE TABLE SQL
- Does NOT load data

### Test 2: Basic Load with Table Creation
```bash
./target/release/csv-sql-loader \
  --create-table \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Analyzes CSV file
- Shows inferred schema
- Creates table
- Loads data
- Shows throughput and timing

### Test 3: Drop and Reload
```bash
./target/release/csv-sql-loader \
  --drop-table \
  --create-table \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Drops existing table
- Creates new table
- Loads data successfully

### Test 4: Custom Table Name
```bash
./target/release/csv-sql-loader \
  --table my_users \
  --create-table \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Creates table named "my_users"
- Loads data

### Test 5: Custom Batch Size
```bash
./target/release/csv-sql-loader \
  --create-table \
  --batch-size 2 \
  --table batch_test \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Processes in batches of 2 rows
- Shows progress updates

### Test 6: Verbose Mode
```bash
./target/release/csv-sql-loader \
  --verbose \
  --create-table \
  --table verbose_test \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Shows detailed logging
- Debug information

### Test 7: Quiet Mode
```bash
./target/release/csv-sql-loader \
  --quiet \
  --create-table \
  --table quiet_test \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected Output:**
- Minimal output
- No progress bar

### Test 8: Help and Version
```bash
# Show help
./target/release/csv-sql-loader --help

# Show version
./target/release/csv-sql-loader --version
```

## Verification Commands

After loading data, verify with PostgreSQL:

```bash
# Connect to database
psql "postgresql://testuser:testpass@localhost:5432/testdb"

# Check tables
\dt

# View table structure
\d sample

# Count rows
SELECT COUNT(*) FROM sample;

# View data
SELECT * FROM sample;

# Exit
\q
```

## What to Capture for Full Testing Report

### For Each Test, Capture:

1. **Command Run:**
   ```
   ./target/release/csv-sql-loader [options] file.csv connection_string
   ```

2. **Console Output:**
   - Complete stdout/stderr
   - Schema inference results
   - Progress information
   - Success/error messages
   - Timing and throughput stats

3. **Database Verification:**
   ```sql
   -- Table structure
   \d table_name

   -- Row count
   SELECT COUNT(*) FROM table_name;

   -- Sample data
   SELECT * FROM table_name LIMIT 5;
   ```

4. **Exit Code:**
   ```bash
   echo $?  # Should be 0 for success
   ```

## Expected Results Summary

| Test | Should Create Table | Should Load Data | Expected Rows |
|------|-------------------|-----------------|---------------|
| Dry Run | No | No | 0 |
| Basic Load | Yes | Yes | 5 |
| Drop & Reload | Yes | Yes | 5 |
| Custom Table | Yes | Yes | 5 |
| Batch Size | Yes | Yes | 5 |
| Verbose | Yes | Yes | 5 |
| Quiet | Yes | Yes | 5 |

## Error Testing

### Test Invalid Connection
```bash
./target/release/csv-sql-loader \
  examples/sample.csv \
  "postgresql://baduser:badpass@localhost:5432/baddb"
```

**Expected:** Connection error with descriptive message

### Test Missing File
```bash
./target/release/csv-sql-loader \
  nonexistent.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected:** File not found error

### Test Existing Table Without --create-table
```bash
# First create table
./target/release/csv-sql-loader \
  --create-table \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"

# Then try again without --create-table or --drop-table
./target/release/csv-sql-loader \
  examples/sample.csv \
  "postgresql://testuser:testpass@localhost:5432/testdb"
```

**Expected:** Table exists, should append data or show warning

## Cleanup

```bash
# Stop and remove Docker container
docker stop test-postgres
docker rm test-postgres

# Or drop database locally
dropdb testdb
```

## Full Test Report Template

When sharing results, provide:

```
=== Test: [Test Name] ===

Command:
[exact command run]

Output:
[complete console output]

Database Check:
[psql output showing table structure and data]

Exit Code: [0 or error code]

Status: PASS/FAIL
Notes: [any observations]

---
```
