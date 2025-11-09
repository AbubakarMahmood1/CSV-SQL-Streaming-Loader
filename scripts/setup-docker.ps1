# PowerShell Docker Setup Script for CSV-SQL Streaming Loader
# Windows version of setup-docker.sh

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "CSV-SQL Streaming Loader - Docker Setup" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Configuration
$CONTAINER_NAME = "csv-sql-loader-postgres"
$POSTGRES_PASSWORD = "csv_loader_pass"
$POSTGRES_USER = "csv_user"
$POSTGRES_DB = "csv_test_db"
$POSTGRES_PORT = "5432"

# Check if Docker is installed
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Docker is not installed" -ForegroundColor Red
    Write-Host "Please install Docker Desktop for Windows:"
    Write-Host "  https://www.docker.com/products/docker-desktop"
    exit 1
}

Write-Host "Checking Docker..." -ForegroundColor Blue
docker --version
Write-Host ""

# Check if container already exists
$existingContainer = docker ps -a --format "{{.Names}}" | Select-String -Pattern "^$CONTAINER_NAME$"

if ($existingContainer) {
    Write-Host "Container '$CONTAINER_NAME' already exists" -ForegroundColor Yellow
    $recreate = Read-Host "Remove and recreate? (y/n)"
    if ($recreate -eq "y") {
        Write-Host "Stopping and removing existing container..."
        docker stop $CONTAINER_NAME 2>$null
        docker rm $CONTAINER_NAME 2>$null
        $existingContainer = $null
    } else {
        Write-Host "Using existing container"
        docker start $CONTAINER_NAME 2>$null
    }
}

if (-not $existingContainer) {
    Write-Host "Creating PostgreSQL container..." -ForegroundColor Blue
    docker run --name $CONTAINER_NAME `
        -e POSTGRES_PASSWORD=$POSTGRES_PASSWORD `
        -e POSTGRES_USER=$POSTGRES_USER `
        -e POSTGRES_DB=$POSTGRES_DB `
        -p "${POSTGRES_PORT}:5432" `
        -d postgres:15

    Write-Host "Waiting for PostgreSQL to start..."
    Start-Sleep -Seconds 5
}

# Wait for PostgreSQL to be ready
Write-Host "Waiting for PostgreSQL to be ready..." -ForegroundColor Blue
for ($i = 1; $i -le 30; $i++) {
    $ready = docker exec $CONTAINER_NAME pg_isready -U $POSTGRES_USER 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "PostgreSQL is ready!" -ForegroundColor Green
        break
    }
    Write-Host "." -NoNewline
    Start-Sleep -Seconds 1
}
Write-Host ""

# Get container IP
$CONTAINER_IP = docker inspect $CONTAINER_NAME -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}'

Write-Host ""
Write-Host "✓ PostgreSQL Setup Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Connection Details:"
Write-Host "  Container Name: $CONTAINER_NAME"
Write-Host "  Database: $POSTGRES_DB"
Write-Host "  User: $POSTGRES_USER"
Write-Host "  Password: $POSTGRES_PASSWORD"
Write-Host "  Port: $POSTGRES_PORT"
Write-Host "  Container IP: $CONTAINER_IP"
Write-Host ""
Write-Host "Connection Strings:"
Write-Host "  From host: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"
Write-Host "  From container: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${CONTAINER_IP}:${POSTGRES_PORT}/${POSTGRES_DB}"
Write-Host ""

# Save connection info to file
@"
`$CONTAINER_NAME = "$CONTAINER_NAME"
`$POSTGRES_USER = "$POSTGRES_USER"
`$POSTGRES_PASSWORD = "$POSTGRES_PASSWORD"
`$POSTGRES_DB = "$POSTGRES_DB"
`$POSTGRES_PORT = "$POSTGRES_PORT"
`$CONTAINER_IP = "$CONTAINER_IP"
"@ | Out-File -FilePath ".docker-connection-info.ps1" -Encoding UTF8

Write-Host "Connection info saved to .docker-connection-info.ps1" -ForegroundColor Green
Write-Host ""
Write-Host "Quick Commands:"
Write-Host "  View logs: docker logs $CONTAINER_NAME"
Write-Host "  Connect: docker exec -it $CONTAINER_NAME psql -U $POSTGRES_USER -d $POSTGRES_DB"
Write-Host "  Stop: docker stop $CONTAINER_NAME"
Write-Host "  Start: docker start $CONTAINER_NAME"
Write-Host "  Remove: docker rm -f $CONTAINER_NAME"
Write-Host ""
