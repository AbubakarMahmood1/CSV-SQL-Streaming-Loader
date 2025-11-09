//! Schema inference from CSV data

use crate::errors::{LoaderError, Result};
use crate::types::SqlType;

/// Column schema with inferred type
#[derive(Debug, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub sql_type: SqlType,
    pub nullable: bool,
    pub sample_count: usize,
    pub null_count: usize,
}

impl ColumnSchema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            sql_type: SqlType::Null,
            nullable: true,
            sample_count: 0,
            null_count: 0,
        }
    }

    /// Update schema with a new value
    pub fn update(&mut self, value: &str) {
        self.sample_count += 1;

        let inferred_type = SqlType::infer_from_str(value);

        if inferred_type == SqlType::Null {
            self.null_count += 1;
        }

        self.sql_type = self.sql_type.merge(&inferred_type);
    }

    /// Finalize the schema after all samples
    pub fn finalize(&mut self) {
        // If all values were null, default to TEXT
        if self.sql_type == SqlType::Null {
            self.sql_type = SqlType::Text;
        }

        // Column is nullable if we saw any nulls
        self.nullable = self.null_count > 0;
    }

    /// Get confidence score (0.0 to 1.0)
    pub fn confidence(&self) -> f64 {
        if self.sample_count == 0 {
            return 0.0;
        }

        // Confidence decreases with null percentage
        let non_null_ratio = 1.0 - (self.null_count as f64 / self.sample_count as f64);

        // TEXT type has lower confidence (could be anything)
        let type_confidence = match self.sql_type {
            SqlType::Text => 0.6,
            SqlType::Null => 0.3,
            _ => 1.0,
        };

        non_null_ratio * type_confidence
    }
}

/// Table schema
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnSchema>,
}

impl TableSchema {
    pub fn new(table_name: String, column_names: Vec<String>) -> Self {
        let columns = column_names
            .into_iter()
            .map(ColumnSchema::new)
            .collect();

        Self {
            table_name,
            columns,
        }
    }

    /// Update all columns with a row of data
    pub fn update_row(&mut self, row: &[String]) -> Result<()> {
        if row.len() != self.columns.len() {
            return Err(LoaderError::SchemaInferenceError(format!(
                "Row has {} columns but schema expects {}",
                row.len(),
                self.columns.len()
            )));
        }

        for (column, value) in self.columns.iter_mut().zip(row.iter()) {
            column.update(value);
        }

        Ok(())
    }

    /// Finalize schema after all samples
    pub fn finalize(&mut self) {
        for column in &mut self.columns {
            column.finalize();
        }
    }

    /// Generate CREATE TABLE SQL statement
    pub fn to_create_table_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.table_name);

        let column_defs: Vec<String> = self.columns
            .iter()
            .map(|col| {
                let nullable = if col.nullable { "" } else { " NOT NULL" };
                format!("  {} {}{}", col.name, col.sql_type.to_sql(), nullable)
            })
            .collect();

        sql.push_str(&column_defs.join(",\n"));
        sql.push_str("\n);");

        sql
    }

    /// Get column names as comma-separated string
    #[allow(dead_code)]
    pub fn column_names(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Validate table name (basic SQL injection prevention)
    pub fn validate_table_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(LoaderError::InvalidTableName("Table name cannot be empty".to_string()));
        }

        // Must start with letter or underscore
        if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
            return Err(LoaderError::InvalidTableName(
                format!("Table name must start with letter or underscore: {}", name)
            ));
        }

        // Only alphanumeric and underscore allowed
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(LoaderError::InvalidTableName(
                format!("Table name contains invalid characters: {}", name)
            ));
        }

        // Reject SQL keywords (basic protection)
        let keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER", "EXEC"];
        if keywords.iter().any(|k| name.eq_ignore_ascii_case(k)) {
            return Err(LoaderError::InvalidTableName(
                format!("Table name cannot be SQL keyword: {}", name)
            ));
        }

        Ok(())
    }
}

/// Schema inference configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InferenceConfig {
    pub sample_size: usize,
    pub has_headers: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            sample_size: 1000,
            has_headers: true,
        }
    }
}

impl InferenceConfig {
    pub fn new(sample_size: usize, has_headers: bool) -> Self {
        Self {
            sample_size,
            has_headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_schema_update() {
        let mut col = ColumnSchema::new("age".to_string());

        col.update("25");
        col.update("30");
        col.update("42");

        assert_eq!(col.sample_count, 3);
        assert_eq!(col.null_count, 0);
    }

    #[test]
    fn test_column_schema_nullable() {
        let mut col = ColumnSchema::new("name".to_string());

        col.update("Alice");
        col.update("");
        col.update("Bob");

        col.finalize();

        assert!(col.nullable);
        assert_eq!(col.null_count, 1);
    }

    #[test]
    fn test_table_schema_create_sql() {
        let mut schema = TableSchema::new(
            "users".to_string(),
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        schema.update_row(&["1".to_string(), "Alice".to_string(), "25".to_string()]).unwrap();
        schema.update_row(&["2".to_string(), "Bob".to_string(), "30".to_string()]).unwrap();

        schema.finalize();

        let sql = schema.to_create_table_sql();
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id SMALLINT NOT NULL"));
        assert!(sql.contains("name TEXT NOT NULL"));
        assert!(sql.contains("age SMALLINT NOT NULL"));
    }

    #[test]
    fn test_validate_table_name() {
        assert!(TableSchema::validate_table_name("users").is_ok());
        assert!(TableSchema::validate_table_name("user_data").is_ok());
        assert!(TableSchema::validate_table_name("_temp").is_ok());

        assert!(TableSchema::validate_table_name("").is_err());
        assert!(TableSchema::validate_table_name("123users").is_err());
        assert!(TableSchema::validate_table_name("user-data").is_err());
        assert!(TableSchema::validate_table_name("SELECT").is_err());
    }
}
