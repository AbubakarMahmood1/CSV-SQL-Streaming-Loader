#!/bin/bash
# Setup local PostgreSQL database for testing

set -e

echo "Setting up local PostgreSQL test database..."
echo ""

# Create test database (if it doesn't exist)
echo "Creating test database 'csv_test_db'..."
psql -U postgres -c "CREATE DATABASE csv_test_db;" 2>/dev/null || \
psql -c "CREATE DATABASE csv_test_db;" 2>/dev/null || \
echo "Database may already exist or using default user"

echo ""
echo "✓ Database setup complete!"
echo ""
echo "Your connection strings:"
echo ""
echo "Option 1 (if using postgres user):"
echo "  postgresql://postgres@localhost/csv_test_db"
echo ""
echo "Option 2 (if using your system user):"
echo "  postgresql://localhost/csv_test_db"
echo ""
echo "Option 3 (with password, if needed):"
echo "  postgresql://USERNAME:PASSWORD@localhost/csv_test_db"
echo ""
