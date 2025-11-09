#!/bin/bash
# Test Runner Script for CSV-SQL Streaming Loader
# Automatically builds and tests the loader with Docker PostgreSQL

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "======================================"
echo "CSV-SQL Streaming Loader - Test Runner"
echo "======================================"
echo ""

# Load connection info if exists
if [ -f .docker-connection-info ]; then
    source .docker-connection-info
    echo -e "${GREEN}Loaded connection info from .docker-connection-info${NC}"
else
    echo -e "${YELLOW}No connection info found. Running setup first...${NC}"
    echo ""
    ./scripts/setup-docker.sh
    source .docker-connection-info
fi

# Check if container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Container not running. Starting...${NC}"
    docker start ${CONTAINER_NAME}
    sleep 3
fi

# Build the project if not already built
if [ ! -f target/release/csv-sql-loader ]; then
    echo -e "${BLUE}Building project...${NC}"
    cargo build --release
    echo ""
fi

# Connection string
CONN_STRING="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"

echo -e "${BLUE}Running Tests...${NC}"
echo ""

# Test 1: Dry Run
echo -e "${BLUE}Test 1: Dry Run (Schema Preview)${NC}"
echo "-----------------------------------"
./target/release/csv-sql-loader --dry-run examples/sample.csv "$CONN_STRING"
echo -e "${GREEN}✓ Dry run passed${NC}"
echo ""

# Test 2: Load Data
echo -e "${BLUE}Test 2: Loading Data${NC}"
echo "-----------------------------------"
./target/release/csv-sql-loader --drop-table --create-table examples/sample.csv "$CONN_STRING"
echo -e "${GREEN}✓ Data loaded${NC}"
echo ""

# Test 3: Verify Data
echo -e "${BLUE}Test 3: Verifying Data${NC}"
echo "-----------------------------------"
echo "Table structure:"
docker exec ${CONTAINER_NAME} psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "\d sample"
echo ""

echo "Row count:"
docker exec ${CONTAINER_NAME} psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "SELECT COUNT(*) as total_rows FROM sample;"
echo ""

echo "Sample data:"
docker exec ${CONTAINER_NAME} psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "SELECT * FROM sample;"
echo ""

echo -e "${GREEN}✓ Data verified${NC}"
echo ""

# Test 4: Custom Table Name
echo -e "${BLUE}Test 4: Custom Table Name${NC}"
echo "-----------------------------------"
./target/release/csv-sql-loader --table custom_users --drop-table --create-table examples/sample.csv "$CONN_STRING"
echo -e "${GREEN}✓ Custom table created${NC}"
echo ""

# Summary
echo "======================================"
echo -e "${GREEN}✓ All Tests Passed!${NC}"
echo "======================================"
echo ""
echo "Tables created:"
docker exec ${CONTAINER_NAME} psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "\dt"
echo ""
