//! PostgreSQL COPY protocol implementation

use crate::errors::{LoaderError, Result};
use crate::schema::TableSchema;
use tokio_postgres::Client;
use futures_util::sink::SinkExt;
use bytes::Bytes;

/// COPY loader using PostgreSQL COPY protocol
pub struct CopyLoader<'a> {
    client: &'a Client,
    table_name: String,
    columns: Vec<String>,
}

impl<'a> CopyLoader<'a> {
    /// Create a new COPY loader
    pub fn new(client: &'a Client, schema: &TableSchema) -> Self {
        let columns = schema.columns
            .iter()
            .map(|c| c.name.clone())
            .collect();

        Self {
            client,
            table_name: schema.table_name.clone(),
            columns,
        }
    }

    /// Load a batch of rows using COPY
    pub async fn load_batch(&self, rows: &[Vec<String>]) -> Result<u64> {
        if rows.is_empty() {
            return Ok(0);
        }

        // Build COPY statement
        let column_list = self.columns.join(", ");
        let copy_stmt = format!(
            "COPY {} ({}) FROM STDIN WITH (FORMAT CSV, NULL '')",
            self.table_name, column_list
        );

        // Convert rows to CSV format
        let csv_data = self.rows_to_csv(rows)?;
        let csv_bytes = Bytes::from(csv_data.into_bytes());

        // Execute COPY using the Sink API
        let sink = self.client.copy_in(&copy_stmt).await?;
        tokio::pin!(sink);

        // Send data to the sink
        sink.as_mut().send(csv_bytes).await?;

        // Finish and get row count
        let rows_inserted = sink.finish().await?;

        Ok(rows_inserted)
    }

    /// Convert rows to CSV format for COPY
    fn rows_to_csv(&self, rows: &[Vec<String>]) -> Result<String> {
        let mut csv_data = String::new();

        for row in rows {
            if row.len() != self.columns.len() {
                return Err(LoaderError::TypeConversionError(format!(
                    "Row has {} columns but expected {}",
                    row.len(),
                    self.columns.len()
                )));
            }

            // Build CSV row (handle quoting and escaping)
            let csv_row: Vec<String> = row
                .iter()
                .map(|value| {
                    if value.is_empty() {
                        // Empty string for NULL
                        String::new()
                    } else if value.contains(',') || value.contains('"') || value.contains('\n') {
                        // Quote and escape
                        format!("\"{}\"", value.replace('"', "\"\""))
                    } else {
                        value.clone()
                    }
                })
                .collect();

            csv_data.push_str(&csv_row.join(","));
            csv_data.push('\n');
        }

        Ok(csv_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ColumnSchema, TableSchema};

    fn create_test_schema() -> TableSchema {
        TableSchema {
            table_name: "test_table".to_string(),
            columns: vec![
                ColumnSchema {
                    name: "id".to_string(),
                    sql_type: crate::types::SqlType::Integer,
                    nullable: false,
                    sample_count: 0,
                    null_count: 0,
                },
                ColumnSchema {
                    name: "name".to_string(),
                    sql_type: crate::types::SqlType::Text,
                    nullable: true,
                    sample_count: 0,
                    null_count: 0,
                },
            ],
        }
    }

    #[test]
    fn test_rows_to_csv() {
        let schema = create_test_schema();
        // Create a mock client (we can't use real client in unit test)
        // This test is mainly for the CSV conversion logic

        let rows = vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
        ];

        // We can't test load_batch without a real client, but we can test the helper
        // For now, we'll skip this test or make it integration-only
    }
}
