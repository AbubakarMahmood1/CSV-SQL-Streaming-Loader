# Connecting to Windows PostgreSQL from WSL

You're running the CSV-SQL Loader in **WSL (Ubuntu)**, but your PostgreSQL is installed on **Windows**.

## Quick Setup

### Step 1: Find Your Windows PostgreSQL Connection Info

On Windows, check:
1. **Username**: Usually `postgres` or your Windows username
2. **Password**: What you set during PostgreSQL installation
3. **Port**: Usually `5432`
4. **Database**: Create a test database or use an existing one

### Step 2: Find Your Windows Host IP (from WSL)

Run this in WSL to find your Windows IP:
```bash
ip route | grep default | awk '{print $3}'
```

Or use the hostname:
```bash
# WSL2 can often access Windows via hostname
hostname.exe | tr -d '\r'
```

### Step 3: Configure Windows PostgreSQL to Accept WSL Connections

On **Windows**, you need to edit PostgreSQL config files:

1. **Find PostgreSQL data directory** (usually):
   - `C:\Program Files\PostgreSQL\16\data\`
   - Or check pgAdmin

2. **Edit `postgresql.conf`**:
   ```
   listen_addresses = '*'    # Or 'localhost,YOUR_WSL_IP'
   port = 5432
   ```

3. **Edit `pg_hba.conf`** (add this line):
   ```
   # Allow connections from WSL
   host    all             all             172.16.0.0/12           md5
   ```

4. **Restart PostgreSQL** (Windows Services or):
   ```powershell
   # In Windows PowerShell (as Administrator)
   Restart-Service postgresql-x64-16
   ```

### Step 4: Create Test Database (on Windows)

Using pgAdmin or psql on Windows:
```sql
CREATE DATABASE csv_test_db;
```

### Step 5: Test Connection from WSL

```bash
# Replace with your actual Windows IP and credentials
psql -h 172.X.X.X -U postgres -d csv_test_db -c "SELECT version();"

# Or if hostname works:
psql -h $(hostname.exe | tr -d '\r') -U postgres -d csv_test_db -c "SELECT version();"
```

### Step 6: Run CSV Loader

```bash
# Use your Windows PostgreSQL connection
./target/release/csv-sql-loader \
  --dry-run \
  examples/sample.csv \
  "postgresql://postgres:YOUR_PASSWORD@WINDOWS_IP:5432/csv_test_db"
```

## Connection String Formats

```bash
# Format 1: With password
postgresql://USERNAME:PASSWORD@WINDOWS_IP:5432/DATABASE

# Format 2: Will prompt for password
postgresql://USERNAME@WINDOWS_IP:5432/DATABASE

# Examples:
# postgresql://postgres:mypass@172.20.144.1:5432/csv_test_db
# postgresql://postgres:mypass@$(hostname.exe | tr -d '\r'):5432/csv_test_db
```

## Alternative: Use Docker in WSL

If connecting to Windows PostgreSQL is complex, just use Docker in WSL:

```bash
# Install Docker in WSL (if not installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Start PostgreSQL in Docker
docker run --name test-postgres \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_DB=csv_test_db \
  -p 5432:5432 \
  -d postgres:15

# Connection string:
postgresql://testuser:testpass@localhost:5432/csv_test_db
```

## Troubleshooting

### Connection refused?
- Check Windows Firewall allows port 5432
- Check PostgreSQL is running on Windows
- Verify `listen_addresses` in postgresql.conf

### Authentication failed?
- Double-check username and password
- Check `pg_hba.conf` has the right entry
- Try `md5` or `scram-sha-256` authentication

### Can't find Windows IP?
```bash
# From WSL, try:
cat /etc/resolv.conf | grep nameserver
ip route show | grep default
```

### Still stuck?
Just use Docker in WSL - it's simpler and isolated!
