#!/bin/bash
# Test connection to Windows PostgreSQL from WSL

echo "======================================"
echo "WSL to Windows PostgreSQL Connector"
echo "======================================"
echo ""

# Detect Windows host
echo "Detecting Windows host IP..."
WINDOWS_IP=$(cat /etc/resolv.conf 2>/dev/null | grep nameserver | awk '{print $2}')

if [ -z "$WINDOWS_IP" ]; then
    echo "Could not auto-detect. Please enter your Windows IP manually."
    echo "(Check in Windows: ipconfig | findstr IPv4)"
    read -p "Windows IP: " WINDOWS_IP
fi

echo "Using Windows IP: $WINDOWS_IP"
echo ""

# Get credentials
read -p "PostgreSQL username [postgres]: " PG_USER
PG_USER=${PG_USER:-postgres}

read -sp "PostgreSQL password: " PG_PASS
echo ""

read -p "Database name [csv_test_db]: " PG_DB
PG_DB=${PG_DB:-csv_test_db}

read -p "Port [5432]: " PG_PORT
PG_PORT=${PG_PORT:-5432}

# Build connection string
CONN_STRING="postgresql://${PG_USER}:${PG_PASS}@${WINDOWS_IP}:${PG_PORT}/${PG_DB}"

echo ""
echo "Testing connection..."
echo "Connection string: postgresql://${PG_USER}:***@${WINDOWS_IP}:${PG_PORT}/${PG_DB}"
echo ""

# Test connection
psql "$CONN_STRING" -c "SELECT version();" 2>&1

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Connection successful!"
    echo ""
    echo "Running CSV Loader dry-run..."
    echo ""
    ./target/release/csv-sql-loader --dry-run examples/sample.csv "$CONN_STRING"
    echo ""
    read -p "Load data for real? (y/n): " CONFIRM
    if [ "$CONFIRM" = "y" ]; then
        ./target/release/csv-sql-loader --drop-table --create-table examples/sample.csv "$CONN_STRING"
        echo ""
        echo "Verifying data..."
        psql "$CONN_STRING" -c "SELECT * FROM sample;"
    fi
else
    echo ""
    echo "✗ Connection failed!"
    echo ""
    echo "Troubleshooting:"
    echo "1. Make sure PostgreSQL is running on Windows"
    echo "2. Check pg_hba.conf allows connections from WSL"
    echo "3. Check Windows Firewall allows port $PG_PORT"
    echo "4. Verify credentials are correct"
    echo ""
    echo "See WINDOWS-POSTGRESQL.md for detailed setup instructions"
fi
