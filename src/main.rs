//! CSV-SQL Streaming Loader
//! High-performance CLI tool for loading CSV files into PostgreSQL

mod errors;
mod types;
mod schema;
mod parser;
mod db;
mod progress;

use clap::Parser;
use errors::{LoaderError, Result};
use parser::CsvParser;
use schema::{InferenceConfig, TableSchema};
use db::{DbConnection, CopyLoader, BatchProcessor, batch::BatchConfig, batch::BatchIterator};
use progress::ProgressTracker;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "csv-sql-loader")]
#[command(version = "0.1.0")]
#[command(about = "High-performance CSV to PostgreSQL loader", long_about = None)]
struct Args {
    /// CSV file to load
    #[arg(value_name = "CSV_FILE")]
    csv_file: PathBuf,

    /// PostgreSQL connection string
    #[arg(value_name = "CONNECTION_STRING")]
    connection_string: String,

    /// Target table name (default: inferred from filename)
    #[arg(short, long)]
    table: Option<String>,

    /// Rows per batch
    #[arg(short, long, default_value_t = 10000)]
    batch_size: usize,

    /// Rows to sample for type inference
    #[arg(short, long, default_value_t = 1000)]
    sample_size: usize,

    /// Create table if it doesn't exist
    #[arg(long)]
    create_table: bool,

    /// Drop table before loading
    #[arg(long)]
    drop_table: bool,

    /// CSV delimiter
    #[arg(short, long, default_value = ",")]
    delimiter: String,

    /// CSV has no header row
    #[arg(long)]
    no_header: bool,

    /// Maximum retry attempts
    #[arg(long, default_value_t = 3)]
    max_retries: usize,

    /// Show inferred schema without loading (dry run)
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress progress display
    #[arg(short, long)]
    quiet: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose);

    // Validate inputs
    if !args.csv_file.exists() {
        return Err(LoaderError::FileNotFound(
            args.csv_file.display().to_string()
        ));
    }

    // Determine table name
    let table_name = args.table.unwrap_or_else(|| {
        args.csv_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_data")
            .to_string()
    });

    TableSchema::validate_table_name(&table_name)?;

    // Parse delimiter
    let delimiter = parser::parse_delimiter(&args.delimiter)?;

    // Parse CSV and infer schema
    let has_headers = !args.no_header;
    let mut parser = CsvParser::from_path(&args.csv_file, delimiter, has_headers)?;

    println!("Analyzing CSV file: {}", args.csv_file.display());

    let inference_config = InferenceConfig::new(args.sample_size, has_headers);
    let schema = parser.infer_schema(table_name.clone(), &inference_config)?;

    // Display schema
    println!("\nInferred Schema:");
    println!("Table: {}", schema.table_name);
    println!("Columns:");
    for col in &schema.columns {
        let nullable = if col.nullable { "NULL" } else { "NOT NULL" };
        let confidence = (col.confidence() * 100.0) as u8;
        println!(
            "  - {} {} {} ({}% confidence, {} samples, {} nulls)",
            col.name,
            col.sql_type.to_sql(),
            nullable,
            confidence,
            col.sample_count,
            col.null_count
        );
    }
    println!();

    // Dry run - exit after showing schema
    if args.dry_run {
        println!("CREATE TABLE SQL:");
        println!("{}", schema.to_create_table_sql());
        println!("\nDry run complete. No data loaded.");
        return Ok(());
    }

    // Connect to database
    println!("Connecting to database...");
    let db = DbConnection::connect(&args.connection_string).await?;

    // Handle table creation/dropping
    if args.drop_table {
        println!("Dropping existing table...");
        db.drop_table(&table_name).await?;
    }

    let table_exists = db.table_exists(&table_name).await?;

    if !table_exists {
        if args.create_table {
            println!("Creating table...");
            let create_sql = schema.to_create_table_sql();
            db.create_table(&create_sql).await?;
        } else {
            return Err(LoaderError::ConfigError(format!(
                "Table '{}' does not exist. Use --create-table to create it.",
                table_name
            )));
        }
    }

    // Reset parser to beginning of file
    parser.reset(&args.csv_file, has_headers)?;

    // Set up batch processor
    let batch_config = BatchConfig {
        batch_size: args.batch_size,
        max_retries: args.max_retries,
        ..Default::default()
    };
    let batch_processor = BatchProcessor::new(batch_config);

    // Set up progress tracker
    let progress = ProgressTracker::new(None, args.quiet);

    // Load data
    println!("Loading data...");

    let loader = CopyLoader::new(db.client(), &schema);
    let mut total_rows = 0u64;

    // Process batches
    let records = parser.records();
    let batches = BatchIterator::new(records, args.batch_size);

    for batch_result in batches {
        let batch = batch_result?;
        let batch_size = batch.len() as u64;

        match batch_processor.process_batch(&loader, batch).await {
            Ok(count) => {
                total_rows += count;
                progress.inc(batch_size);
            }
            Err(e) => {
                progress.finish_with_error(&e.to_string());
                return Err(e);
            }
        }
    }

    progress.finish();

    println!("\n✓ Successfully loaded {} rows into '{}'", total_rows, table_name);
    println!("  Throughput: {:.0} rows/sec", progress.throughput());
    println!("  Time: {:.2}s", progress.elapsed().as_secs_f64());

    Ok(())
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = if verbose {
        EnvFilter::new("csv_sql_loader=debug")
    } else {
        EnvFilter::new("csv_sql_loader=info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
