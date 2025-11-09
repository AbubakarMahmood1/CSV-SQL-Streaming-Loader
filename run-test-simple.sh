#!/bin/bash
# Simplest possible test - paste your connection string when prompted

echo "CSV-SQL Loader Simple Test"
echo "=========================="
echo ""
echo "First, make sure PostgreSQL is running:"
echo "  sudo service postgresql start"
echo ""
read -p "Enter your PostgreSQL connection string: " CONN_STRING

if [ -z "$CONN_STRING" ]; then
    echo "Using default: postgresql://postgres@localhost/csv_test_db"
    CONN_STRING="postgresql://postgres@localhost/csv_test_db"
fi

echo ""
echo "Testing connection..."
psql "$CONN_STRING" -c "SELECT 1;" 2>&1

if [ $? -eq 0 ]; then
    echo "✓ Connection works!"
    echo ""
    echo "Running CSV Loader..."
    ./target/release/csv-sql-loader --drop-table --create-table examples/sample.csv "$CONN_STRING"
else
    echo "✗ Connection failed. Try:"
    echo "  - Start PostgreSQL: sudo service postgresql start"
    echo "  - Create database: sudo -u postgres createdb csv_test_db"
    echo "  - Try different connection string"
fi
