# Quick Start - Testing with Your Local PostgreSQL

## Step 1: Start PostgreSQL

Your PostgreSQL is installed but not currently running. Start it with:

```bash
# Option A: Using systemd (most common)
sudo service postgresql start

# Option B: Check status
sudo service postgresql status
```

## Step 2: Create Test Database

```bash
# Connect as postgres user and create test database
sudo -u postgres createdb csv_test_db

# Or create it directly in psql:
sudo -u postgres psql -c "CREATE DATABASE csv_test_db;"
```

## Step 3: Find Your Connection String

Most likely one of these will work:

```bash
# Try these in order:
postgresql://postgres@localhost/csv_test_db
postgresql://postgres@localhost:5432/csv_test_db
postgresql:///csv_test_db
```

## Step 4: Run Quick Test

```bash
# Test the connection first
psql -U postgres -d csv_test_db -c "SELECT version();"

# If that works, run the CSV loader:
./target/release/csv-sql-loader \
  --dry-run \
  examples/sample.csv \
  "postgresql://postgres@localhost/csv_test_db"
```

## Step 5: Full Test

Once dry-run works, load the data:

```bash
./target/release/csv-sql-loader \
  --drop-table \
  --create-table \
  examples/sample.csv \
  "postgresql://postgres@localhost/csv_test_db"
```

## Step 6: Verify Data

```bash
# Connect to database
psql -U postgres -d csv_test_db

# Inside psql:
\dt                    -- List tables
\d sample             -- Show table structure
SELECT * FROM sample; -- View data
SELECT COUNT(*) FROM sample; -- Count rows (should be 5)
\q                    -- Quit
```

---

## Automated Script (After PostgreSQL is Running)

Once PostgreSQL is running, you can use the automated script:

```bash
./test-local.sh "postgresql://postgres@localhost/csv_test_db"
```

---

## Troubleshooting

### PostgreSQL won't start?
```bash
# Check if it's installed
dpkg -l | grep postgresql

# Check the service
sudo systemctl status postgresql

# Start it
sudo systemctl start postgresql
```

### Connection refused?
```bash
# Check if PostgreSQL is listening
sudo netstat -nlpt | grep postgres

# Check PostgreSQL config
sudo cat /etc/postgresql/*/main/postgresql.conf | grep listen_addresses
```

### Permission denied?
```bash
# You might need to connect as the postgres user
sudo -u postgres psql

# Then inside psql, create your database:
CREATE DATABASE csv_test_db;
\q
```

---

## Alternative: Use Docker (If Local PostgreSQL Issues)

If you have trouble with local PostgreSQL, use Docker instead:

```bash
docker run --name test-postgres \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_DB=csv_test_db \
  -p 5432:5432 \
  -d postgres:15

# Connection string:
# postgresql://testuser:testpass@localhost:5432/csv_test_db
```

---

## What to Share for Testing Report

After running tests, copy and paste this to me:

```bash
# 1. PostgreSQL Version
psql --version

# 2. Test Run Output
./target/release/csv-sql-loader --dry-run examples/sample.csv "YOUR_CONNECTION_STRING"

# 3. Actual Load
./target/release/csv-sql-loader --drop-table --create-table examples/sample.csv "YOUR_CONNECTION_STRING"

# 4. Database Verification
psql "YOUR_CONNECTION_STRING" -c "\d sample"
psql "YOUR_CONNECTION_STRING" -c "SELECT * FROM sample;"
psql "YOUR_CONNECTION_STRING" -c "SELECT COUNT(*) FROM sample;"
```

Copy **all output** from above and share with me!
