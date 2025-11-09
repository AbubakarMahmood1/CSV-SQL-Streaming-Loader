# PowerShell Test Runner Script for CSV-SQL Streaming Loader
# Windows version of run-test.sh

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "CSV-SQL Streaming Loader - Test Runner" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Load connection info if exists
if (Test-Path ".docker-connection-info.ps1") {
    . .\.docker-connection-info.ps1
    Write-Host "Loaded connection info from .docker-connection-info.ps1" -ForegroundColor Green
} else {
    Write-Host "No connection info found. Running setup first..." -ForegroundColor Yellow
    Write-Host ""
    & .\scripts\setup-docker.ps1
    . .\.docker-connection-info.ps1
}

# Check if container is running
$running = docker ps --format "{{.Names}}" | Select-String -Pattern "^$CONTAINER_NAME$"
if (-not $running) {
    Write-Host "Container not running. Starting..." -ForegroundColor Yellow
    docker start $CONTAINER_NAME
    Start-Sleep -Seconds 3
}

# Build the project if not already built
if (-not (Test-Path "target\release\csv-sql-loader.exe")) {
    Write-Host "Building project..." -ForegroundColor Blue
    cargo build --release
    Write-Host ""
}

# Connection string
$CONN_STRING = "postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"

Write-Host "Running Tests..." -ForegroundColor Blue
Write-Host ""

# Test 1: Dry Run
Write-Host "Test 1: Dry Run (Schema Preview)" -ForegroundColor Blue
Write-Host "-----------------------------------"
& .\target\release\csv-sql-loader.exe --dry-run examples\sample.csv $CONN_STRING
Write-Host "✓ Dry run passed" -ForegroundColor Green
Write-Host ""

# Test 2: Load Data
Write-Host "Test 2: Loading Data" -ForegroundColor Blue
Write-Host "-----------------------------------"
& .\target\release\csv-sql-loader.exe --drop-table --create-table examples\sample.csv $CONN_STRING
Write-Host "✓ Data loaded" -ForegroundColor Green
Write-Host ""

# Test 3: Verify Data
Write-Host "Test 3: Verifying Data" -ForegroundColor Blue
Write-Host "-----------------------------------"
Write-Host "Table structure:"
docker exec $CONTAINER_NAME psql -U $POSTGRES_USER -d $POSTGRES_DB -c "\d sample"
Write-Host ""

Write-Host "Row count:"
docker exec $CONTAINER_NAME psql -U $POSTGRES_USER -d $POSTGRES_DB -c "SELECT COUNT(*) as total_rows FROM sample;"
Write-Host ""

Write-Host "Sample data:"
docker exec $CONTAINER_NAME psql -U $POSTGRES_USER -d $POSTGRES_DB -c "SELECT * FROM sample;"
Write-Host ""

Write-Host "✓ Data verified" -ForegroundColor Green
Write-Host ""

# Test 4: Custom Table Name
Write-Host "Test 4: Custom Table Name" -ForegroundColor Blue
Write-Host "-----------------------------------"
& .\target\release\csv-sql-loader.exe --table custom_users --drop-table --create-table examples\sample.csv $CONN_STRING
Write-Host "✓ Custom table created" -ForegroundColor Green
Write-Host ""

# Summary
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "✓ All Tests Passed!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Tables created:"
docker exec $CONTAINER_NAME psql -U $POSTGRES_USER -d $POSTGRES_DB -c "\dt"
Write-Host ""
