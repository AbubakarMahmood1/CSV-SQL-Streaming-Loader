#!/bin/bash
# Quick test with local PostgreSQL

# Default connection string - modify if needed
CONN_STRING="${1:-postgresql://localhost/csv_test_db}"

echo "==================================="
echo "Testing CSV-SQL Loader with Local PostgreSQL"
echo "==================================="
echo ""
echo "Connection: $CONN_STRING"
echo ""

# Test 1: Dry run
echo "Test 1: Dry Run (Schema Preview)"
echo "---------------------------------"
./target/release/csv-sql-loader --dry-run examples/sample.csv "$CONN_STRING"
echo ""
read -p "Press Enter to continue to actual load..."
echo ""

# Test 2: Load data
echo "Test 2: Loading Data"
echo "---------------------------------"
./target/release/csv-sql-loader --drop-table --create-table examples/sample.csv "$CONN_STRING"
echo ""
echo "✓ Load complete!"
echo ""

# Test 3: Verify in database
echo "Test 3: Verifying Data in Database"
echo "---------------------------------"
psql "$CONN_STRING" -c "\dt"
echo ""
psql "$CONN_STRING" -c "\d sample"
echo ""
psql "$CONN_STRING" -c "SELECT * FROM sample;"
echo ""
psql "$CONN_STRING" -c "SELECT COUNT(*) as total_rows FROM sample;"
echo ""

echo "==================================="
echo "✓ All Tests Complete!"
echo "==================================="
