#!/bin/bash
# Docker Setup Script for CSV-SQL Streaming Loader
# This script automatically sets up PostgreSQL in Docker and tests the loader

set -e

echo "======================================"
echo "CSV-SQL Streaming Loader - Docker Setup"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
CONTAINER_NAME="csv-sql-loader-postgres"
POSTGRES_PASSWORD="csv_loader_pass"
POSTGRES_USER="csv_user"
POSTGRES_DB="csv_test_db"
POSTGRES_PORT="5432"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    echo "Please install Docker first:"
    echo "  - Windows/Mac: https://www.docker.com/products/docker-desktop"
    echo "  - Linux: https://docs.docker.com/engine/install/"
    exit 1
fi

echo -e "${BLUE}Checking Docker...${NC}"
docker --version
echo ""

# Check if container already exists
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Container '${CONTAINER_NAME}' already exists${NC}"
    read -p "Remove and recreate? (y/n): " RECREATE
    if [ "$RECREATE" = "y" ]; then
        echo "Stopping and removing existing container..."
        docker stop ${CONTAINER_NAME} 2>/dev/null || true
        docker rm ${CONTAINER_NAME} 2>/dev/null || true
    else
        echo "Using existing container"
        docker start ${CONTAINER_NAME} 2>/dev/null || true
    fi
else
    echo -e "${BLUE}Creating PostgreSQL container...${NC}"
    docker run --name ${CONTAINER_NAME} \
        -e POSTGRES_PASSWORD=${POSTGRES_PASSWORD} \
        -e POSTGRES_USER=${POSTGRES_USER} \
        -e POSTGRES_DB=${POSTGRES_DB} \
        -p ${POSTGRES_PORT}:5432 \
        -d postgres:15

    echo "Waiting for PostgreSQL to start..."
    sleep 5
fi

# Wait for PostgreSQL to be ready
echo -e "${BLUE}Waiting for PostgreSQL to be ready...${NC}"
for i in {1..30}; do
    if docker exec ${CONTAINER_NAME} pg_isready -U ${POSTGRES_USER} > /dev/null 2>&1; then
        echo -e "${GREEN}PostgreSQL is ready!${NC}"
        break
    fi
    echo -n "."
    sleep 1
done
echo ""

# Get container IP
CONTAINER_IP=$(docker inspect ${CONTAINER_NAME} -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')

echo ""
echo -e "${GREEN}✓ PostgreSQL Setup Complete!${NC}"
echo ""
echo "Connection Details:"
echo "  Container Name: ${CONTAINER_NAME}"
echo "  Database: ${POSTGRES_DB}"
echo "  User: ${POSTGRES_USER}"
echo "  Password: ${POSTGRES_PASSWORD}"
echo "  Port: ${POSTGRES_PORT}"
echo "  Container IP: ${CONTAINER_IP}"
echo ""
echo "Connection Strings:"
echo "  From host: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"
echo "  From container: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${CONTAINER_IP}:${POSTGRES_PORT}/${POSTGRES_DB}"
echo ""

# Save connection info to file
cat > .docker-connection-info <<EOF
CONTAINER_NAME=${CONTAINER_NAME}
POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=${POSTGRES_DB}
POSTGRES_PORT=${POSTGRES_PORT}
CONTAINER_IP=${CONTAINER_IP}
EOF

echo -e "${GREEN}Connection info saved to .docker-connection-info${NC}"
echo ""
echo "Quick Commands:"
echo "  View logs: docker logs ${CONTAINER_NAME}"
echo "  Connect: docker exec -it ${CONTAINER_NAME} psql -U ${POSTGRES_USER} -d ${POSTGRES_DB}"
echo "  Stop: docker stop ${CONTAINER_NAME}"
echo "  Start: docker start ${CONTAINER_NAME}"
echo "  Remove: docker rm -f ${CONTAINER_NAME}"
echo ""
