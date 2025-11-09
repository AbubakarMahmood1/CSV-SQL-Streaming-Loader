#!/bin/bash
# Quick Test Script for CSV-SQL Streaming Loader

set -e

echo "==================================="
echo "CSV-SQL Streaming Loader - Quick Test"
echo "==================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if PostgreSQL connection string is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: PostgreSQL connection string required${NC}"
    echo ""
    echo "Usage: ./quick-test.sh 'postgresql://user:pass@localhost:5432/dbname'"
    echo ""
    echo "Example with Docker:"
    echo "  docker run --name test-postgres -e POSTGRES_PASSWORD=testpass -e POSTGRES_USER=testuser -e POSTGRES_DB=testdb -p 5432:5432 -d postgres:15"
    echo "  ./quick-test.sh 'postgresql://testuser:testpass@localhost:5432/testdb'"
    exit 1
fi

CONN_STRING="$1"
BINARY="./target/release/csv-sql-loader"

# Build if needed
if [ ! -f "$BINARY" ]; then
    echo -e "${BLUE}Building release binary...${NC}"
    cargo build --release
    echo ""
fi

echo -e "${BLUE}Test 1: Dry Run (Schema Preview)${NC}"
echo "Command: $BINARY --dry-run examples/sample.csv \"$CONN_STRING\""
echo "---"
$BINARY --dry-run examples/sample.csv "$CONN_STRING"
echo ""
echo -e "${GREEN}✓ Test 1 Complete${NC}"
echo ""
sleep 2

echo -e "${BLUE}Test 2: Load Data with Table Creation${NC}"
echo "Command: $BINARY --drop-table --create-table examples/sample.csv \"$CONN_STRING\""
echo "---"
$BINARY --drop-table --create-table examples/sample.csv "$CONN_STRING"
echo ""
echo -e "${GREEN}✓ Test 2 Complete${NC}"
echo ""
sleep 2

echo -e "${BLUE}Test 3: Custom Table Name${NC}"
echo "Command: $BINARY --table my_test_users --drop-table --create-table examples/sample.csv \"$CONN_STRING\""
echo "---"
$BINARY --table my_test_users --drop-table --create-table examples/sample.csv "$CONN_STRING"
echo ""
echo -e "${GREEN}✓ Test 3 Complete${NC}"
echo ""
sleep 2

echo -e "${BLUE}Test 4: Verbose Mode${NC}"
echo "Command: $BINARY --table verbose_test --drop-table --create-table --verbose examples/sample.csv \"$CONN_STRING\""
echo "---"
$BINARY --table verbose_test --drop-table --create-table --verbose examples/sample.csv "$CONN_STRING"
echo ""
echo -e "${GREEN}✓ Test 4 Complete${NC}"
echo ""

echo "==================================="
echo -e "${GREEN}All Tests Passed!${NC}"
echo "==================================="
echo ""
echo "Tables created:"
echo "  - sample (default name)"
echo "  - my_test_users"
echo "  - verbose_test"
echo ""
echo "To verify, run:"
echo "  psql \"$CONN_STRING\" -c '\\dt'"
echo "  psql \"$CONN_STRING\" -c 'SELECT * FROM sample;'"
echo ""
