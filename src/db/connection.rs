//! Database connection management

use crate::errors::{LoaderError, Result};
use tokio_postgres::{Client, NoTls};

/// Database connection wrapper
pub struct DbConnection {
    client: Client,
}

impl DbConnection {
    /// Connect to PostgreSQL database
    pub async fn connect(connection_string: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await
            .map_err(|e| LoaderError::ConnectionError(e.to_string()))?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(Self { client })
    }

    /// Get reference to client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Execute a simple SQL statement
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.client
            .execute(sql, &[])
            .await
            .map_err(Into::into)
    }

    /// Check if table exists
    pub async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let query = "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = $1
        )";

        let row = self.client
            .query_one(query, &[&table_name])
            .await?;

        Ok(row.get(0))
    }

    /// Create table from SQL
    pub async fn create_table(&self, create_sql: &str) -> Result<()> {
        self.execute(create_sql).await?;
        Ok(())
    }

    /// Drop table if exists
    pub async fn drop_table(&self, table_name: &str) -> Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", table_name);
        self.execute(&sql).await?;
        Ok(())
    }

    /// Begin transaction
    pub async fn begin_transaction(&self) -> Result<()> {
        self.execute("BEGIN").await?;
        Ok(())
    }

    /// Commit transaction
    pub async fn commit_transaction(&self) -> Result<()> {
        self.execute("COMMIT").await?;
        Ok(())
    }

    /// Rollback transaction
    pub async fn rollback_transaction(&self) -> Result<()> {
        self.execute("ROLLBACK").await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL instance
    // They are marked as ignored by default

    #[tokio::test]
    #[ignore]
    async fn test_connection() {
        let conn = DbConnection::connect("postgresql://localhost/test").await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_table_operations() {
        let conn = DbConnection::connect("postgresql://localhost/test")
            .await
            .unwrap();

        conn.drop_table("test_table").await.unwrap();

        let exists = conn.table_exists("test_table").await.unwrap();
        assert!(!exists);

        conn.create_table("CREATE TABLE test_table (id INTEGER)")
            .await
            .unwrap();

        let exists = conn.table_exists("test_table").await.unwrap();
        assert!(exists);

        conn.drop_table("test_table").await.unwrap();
    }
}
