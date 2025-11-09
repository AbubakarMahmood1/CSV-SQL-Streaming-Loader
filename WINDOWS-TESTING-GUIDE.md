# Windows Testing Guide

Since you're accessing Claude Code via web browser, you need to pull this code to your local Windows machine to test with your PostgreSQL installation.

## Step 1: Pull the Code to Windows

### Option A: Using Git Bash / PowerShell

```bash
# Clone the repo (if you haven't already)
git clone https://github.com/AbubakarMahmood1/CSV-SQL-Streaming-Loader.git
cd CSV-SQL-Streaming-Loader

# Or pull latest changes (if already cloned)
git pull origin main
```

### Option B: Download from GitHub

1. Go to: https://github.com/AbubakarMahmood1/CSV-SQL-Streaming-Loader
2. Click **Code** → **Download ZIP**
3. Extract to a folder

---

## Step 2: Install Rust on Windows

### Using winget (Windows 11/10)
```powershell
winget install Rustlang.Rustup
```

### Or download from:
https://rustup.rs/

### Verify installation:
```powershell
rustc --version
cargo --version
```

---

## Step 3: Build the Project

```powershell
# Navigate to project directory
cd CSV-SQL-Streaming-Loader

# Build release binary
cargo build --release

# The binary will be at: target\release\csv-sql-loader.exe
```

---

## Step 4: Prepare Your PostgreSQL

### Find your PostgreSQL connection details:

1. **Username**: Usually `postgres` or your Windows username
2. **Password**: What you set during installation
3. **Database**: Create a test database

### Create test database:

**Using pgAdmin:**
- Right-click Databases → Create → Database
- Name: `csv_test_db`

**Using psql:**
```sql
CREATE DATABASE csv_test_db;
```

---

## Step 5: Test Dry Run (No Database Changes)

```powershell
# Replace YOUR_PASSWORD with your actual password
.\target\release\csv-sql-loader.exe --dry-run examples\sample.csv "postgresql://postgres:YOUR_PASSWORD@localhost:5432/csv_test_db"
```

**Expected output:**
- Shows inferred schema
- Displays column types
- Shows CREATE TABLE SQL
- **Does NOT load data**

---

## Step 6: Load Data

```powershell
.\target\release\csv-sql-loader.exe --drop-table --create-table examples\sample.csv "postgresql://postgres:YOUR_PASSWORD@localhost:5432/csv_test_db"
```

**Expected output:**
- Analyzes CSV
- Creates table
- Loads 5 rows
- Shows throughput stats

---

## Step 7: Verify in PostgreSQL

### Using pgAdmin:
1. Refresh databases
2. Expand `csv_test_db` → Schemas → public → Tables
3. Right-click `sample` → View/Edit Data → All Rows

### Using psql:
```sql
-- Connect to database
\c csv_test_db

-- List tables
\dt

-- View table structure
\d sample

-- View data
SELECT * FROM sample;

-- Count rows
SELECT COUNT(*) FROM sample;
```

**Expected results:**
- Table `sample` exists
- 6 columns: id, name, age, email, salary, created_at
- 5 rows of data
- Correct data types

---

## Full Test Script (Copy-Paste)

Save this as `test-windows.ps1`:

```powershell
# Windows PowerShell Test Script
$PASSWORD = Read-Host "Enter PostgreSQL password" -AsString
$CONN = "postgresql://postgres:$PASSWORD@localhost:5432/csv_test_db"

Write-Host "`n=== Building Release Binary ===" -ForegroundColor Cyan
cargo build --release

Write-Host "`n=== Test 1: Dry Run ===" -ForegroundColor Cyan
.\target\release\csv-sql-loader.exe --dry-run examples\sample.csv $CONN

Write-Host "`n=== Test 2: Load Data ===" -ForegroundColor Cyan
.\target\release\csv-sql-loader.exe --drop-table --create-table examples\sample.csv $CONN

Write-Host "`n=== Test 3: Verify Data ===" -ForegroundColor Cyan
& "psql" "-d" "csv_test_db" "-c" "SELECT * FROM sample;"

Write-Host "`n=== All Tests Complete! ===" -ForegroundColor Green
```

Run it:
```powershell
.\test-windows.ps1
```

---

## What to Share With Claude

After testing, copy ALL output from:

```powershell
# Run this and copy everything:
.\target\release\csv-sql-loader.exe --version
.\target\release\csv-sql-loader.exe --dry-run examples\sample.csv "YOUR_CONNECTION_STRING"
.\target\release\csv-sql-loader.exe --drop-table --create-table examples\sample.csv "YOUR_CONNECTION_STRING"
psql -d csv_test_db -c "\d sample"
psql -d csv_test_db -c "SELECT * FROM sample;"
```

Paste it all back in the chat!

---

## Troubleshooting

### "cargo: command not found"
- Restart PowerShell after installing Rust
- Or add Rust to PATH: `$env:PATH += ";$env:USERPROFILE\.cargo\bin"`

### "psql: command not found"
- Add PostgreSQL bin to PATH
- Usually at: `C:\Program Files\PostgreSQL\16\bin`

### Connection refused
- Make sure PostgreSQL service is running
- Check Services (services.msc) → postgresql-x64-16

### Authentication failed
- Double-check password
- Check pg_hba.conf allows local connections

---

## Quick Test Checklist

- [ ] Rust installed (`rustc --version`)
- [ ] Project built (`cargo build --release`)
- [ ] PostgreSQL running
- [ ] Test database created
- [ ] Dry run works
- [ ] Data loads successfully
- [ ] Data verifiable in database
- [ ] Output shared with Claude

Once complete, merge the feature branch to main!
