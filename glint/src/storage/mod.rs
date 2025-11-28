mod db_path;
mod schema;

use crate::models::{Log, Metric, Span, Trace};
use duckdb::{Connection, Result as DuckDbResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub use db_path::{detect_project_root, get_config_dir, get_data_dir, get_default_db_path, get_project_db_path};
pub use schema::init_schema;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Main storage interface for Glint
///
/// By default, Glint stores data in a file-based database (`glint.db`) to prevent
/// excessive memory usage for large projects. You can:
/// - Use the default database: `Storage::new()`
/// - Specify a custom path: `Storage::new_with_path("custom.db")`
/// - Delete the database: `Storage::delete_database("glint.db")`
#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    /// Create a new storage instance with automatic project-based database
    ///
    /// This will:
    /// 1. Detect the current project by looking for markers (.git, Cargo.toml, package.json, etc.)
    /// 2. Create a database in ~/.config/glint/<project_name>.db
    /// 3. Multiple terminals in the same project will share the same database
    pub fn new() -> Result<Self> {
        let db_path = get_project_db_path().map_err(|e| {
            StorageError::InvalidInput(format!("Failed to determine database path: {}", e))
        })?;

        Self::new_with_path(&db_path)
    }

    /// Create a new storage instance with an in-memory database (only for testing, no use this in app pls!)
    ///
    /// This is available in test mode for all crates to use
    #[doc(hidden)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create a new storage instance with a custom file path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();

        if let Some(parent) = path_ref.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to create directory: {}", e))
            })?;
        }

        let conn = Connection::open(path)?;
        init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Delete the database file
    pub fn delete_database<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }

    /// Insert a span
    pub fn insert_span(&self, span: &Span) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let attributes_json = serde_json::to_string(&span.attributes)?;
        let status_json = serde_json::to_string(&span.status)?;

        conn.execute(
            "INSERT INTO spans (
                span_id, trace_id, parent_span_id, name, kind,
                start_time_unix_nano, end_time_unix_nano,
                attributes, status, service_name
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                &span.span_id,
                &span.trace_id,
                &span.parent_span_id,
                &span.name,
                format!("{:?}", span.kind),
                span.start_time_unix_nano,
                span.end_time_unix_nano,
                attributes_json,
                status_json,
                &span.service_name,
            ],
        )?;

        Ok(())
    }

    /// Insert multiple spans
    pub fn insert_spans(&self, spans: &[Span]) -> Result<()> {
        for span in spans {
            self.insert_span(span)?;
        }
        Ok(())
    }

    /// Insert a log
    pub fn insert_log(&self, log: &Log) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let attributes_json = serde_json::to_string(&log.attributes)?;

        conn.execute(
            "INSERT INTO logs (
                time_unix_nano, severity_level, severity_text, body,
                attributes, trace_id, span_id, service_name
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                log.time_unix_nano,
                format!("{:?}", log.severity_level),
                &log.severity_text,
                &log.body,
                attributes_json,
                &log.trace_id,
                &log.span_id,
                &log.service_name,
            ],
        )?;

        Ok(())
    }

    /// Insert multiple logs
    pub fn insert_logs(&self, logs: &[Log]) -> Result<()> {
        for log in logs {
            self.insert_log(log)?;
        }
        Ok(())
    }

    /// Insert a metric
    pub fn insert_metric(&self, metric: &Metric) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        for data_point in &metric.data_points {
            let attributes_json = serde_json::to_string(&data_point.attributes)?;

            conn.execute(
                "INSERT INTO metrics (
                    name, description, unit, metric_type, temporality,
                    time_unix_nano, start_time_unix_nano, value,
                    attributes, service_name
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                duckdb::params![
                    &metric.name,
                    &metric.description,
                    &metric.unit,
                    format!("{:?}", metric.metric_type),
                    format!("{:?}", metric.temporality),
                    data_point.time_unix_nano,
                    data_point.start_time_unix_nano,
                    data_point.value,
                    attributes_json,
                    &metric.service_name,
                ],
            )?;
        }

        Ok(())
    }

    /// Insert multiple metrics
    pub fn insert_metrics(&self, metrics: &[Metric]) -> Result<()> {
        for metric in metrics {
            self.insert_metric(metric)?;
        }
        Ok(())
    }

    /// Get a complete trace by ID
    pub fn get_trace_by_id(&self, trace_id: &str) -> Result<Trace> {
        let spans = self.get_spans_by_trace_id(trace_id)?;

        if spans.is_empty() {
            return Err(StorageError::NotFound(format!(
                "Trace not found: {}",
                trace_id
            )));
        }

        Ok(Trace::new(trace_id.to_string(), spans))
    }

    /// Get all spans for a trace
    fn get_spans_by_trace_id(&self, trace_id: &str) -> Result<Vec<Span>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT span_id, trace_id, parent_span_id, name, kind,
                    start_time_unix_nano, end_time_unix_nano,
                    attributes, status, service_name
             FROM spans
             WHERE trace_id = ?
             ORDER BY start_time_unix_nano",
        )?;

        let spans = stmt
            .query_map([trace_id], |row| {
                let attributes_json: String = row.get(7)?;
                let status_json: String = row.get(8)?;

                let attributes = serde_json::from_str(&attributes_json).map_err(|e| {
                    duckdb::Error::FromSqlConversionFailure(
                        7,
                        duckdb::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let status = serde_json::from_str(&status_json).map_err(|e| {
                    duckdb::Error::FromSqlConversionFailure(
                        8,
                        duckdb::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let kind_str: String = row.get(4)?;
                let kind = match kind_str.as_str() {
                    "Unspecified" => crate::models::SpanKind::Unspecified,
                    "Internal" => crate::models::SpanKind::Internal,
                    "Server" => crate::models::SpanKind::Server,
                    "Client" => crate::models::SpanKind::Client,
                    "Producer" => crate::models::SpanKind::Producer,
                    "Consumer" => crate::models::SpanKind::Consumer,
                    _ => crate::models::SpanKind::Unspecified,
                };

                Ok(Span::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    kind,
                    row.get(5)?,
                    row.get(6)?,
                    attributes,
                    status,
                    row.get(9)?,
                ))
            })?
            .collect::<DuckDbResult<Vec<_>>>()?;

        Ok(spans)
    }

    /// List traces with optional filters
    pub fn list_traces(
        &self,
        service_name: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Trace>> {
        let conn = self.conn.lock().unwrap();

        // get a unique ids
        let query = if let Some(service) = service_name {
            format!(
                "SELECT DISTINCT trace_id FROM spans WHERE service_name = '{}' ORDER BY start_time_unix_nano DESC LIMIT {}",
                service,
                limit.unwrap_or(100)
            )
        } else {
            format!(
                "SELECT DISTINCT trace_id FROM spans ORDER BY start_time_unix_nano DESC LIMIT {}",
                limit.unwrap_or(100)
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let trace_ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<DuckDbResult<Vec<_>>>()?;

        drop(stmt);
        drop(conn);

        // full traces
        let mut traces = Vec::new();
        for trace_id in trace_ids {
            if let Ok(trace) = self.get_trace_by_id(&trace_id) {
                traces.push(trace);
            }
        }

        Ok(traces)
    }

    /// List logs with optional filters
    pub fn list_logs(&self, service_name: Option<&str>, limit: Option<usize>) -> Result<Vec<Log>> {
        let conn = self.conn.lock().unwrap();

        let query = if let Some(service) = service_name {
            format!(
                "SELECT time_unix_nano, severity_level, severity_text, body,
                        attributes, trace_id, span_id, service_name
                 FROM logs
                 WHERE service_name = '{}'
                 ORDER BY time_unix_nano DESC
                 LIMIT {}",
                service,
                limit.unwrap_or(100)
            )
        } else {
            format!(
                "SELECT time_unix_nano, severity_level, severity_text, body,
                        attributes, trace_id, span_id, service_name
                 FROM logs
                 ORDER BY time_unix_nano DESC
                 LIMIT {}",
                limit.unwrap_or(100)
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let logs = stmt
            .query_map([], |row| {
                let attributes_json: String = row.get(4)?;
                let attributes = serde_json::from_str(&attributes_json).map_err(|e| {
                    duckdb::Error::FromSqlConversionFailure(
                        4,
                        duckdb::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let severity_str: String = row.get(1)?;
                let severity_level = match severity_str.as_str() {
                    "Info" => crate::models::SeverityLevel::Info,
                    "Warn" => crate::models::SeverityLevel::Warn,
                    "Error" => crate::models::SeverityLevel::Error,
                    "Debug" => crate::models::SeverityLevel::Debug,
                    "Trace" => crate::models::SeverityLevel::Trace,
                    "Fatal" => crate::models::SeverityLevel::Fatal,
                    _ => crate::models::SeverityLevel::Unspecified,
                };

                Ok(Log::new(
                    row.get(0)?,
                    severity_level,
                    row.get(2)?,
                    row.get(3)?,
                    attributes,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .collect::<DuckDbResult<Vec<_>>>()?;

        Ok(logs)
    }

    /// Get count of spans
    pub fn count_spans(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM spans", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get count of logs
    pub fn count_logs(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get count of metrics
    pub fn count_metrics(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Attributes, SpanKind, Status};

    fn create_test_span(span_id: &str, trace_id: &str) -> Span {
        Span::new(
            span_id.to_string(),
            trace_id.to_string(),
            None,
            "test-operation".to_string(),
            SpanKind::Server,
            1_000_000_000_000_000_000,
            1_000_000_000_100_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        )
    }

    #[test]
    fn test_storage_new_in_memory() {
        let storage = Storage::new_in_memory();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_insert_and_get_span() {
        let storage = Storage::new_in_memory().unwrap();
        let span = create_test_span("span1", "trace1");

        storage.insert_span(&span).unwrap();
        let trace = storage.get_trace_by_id("trace1").unwrap();

        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].span_id, "span1");
    }

    #[test]
    fn test_insert_multiple_spans() {
        let storage = Storage::new_in_memory().unwrap();
        let spans = vec![
            create_test_span("span1", "trace1"),
            create_test_span("span2", "trace1"),
        ];

        storage.insert_spans(&spans).unwrap();
        let trace = storage.get_trace_by_id("trace1").unwrap();

        assert_eq!(trace.spans.len(), 2);
    }

    #[test]
    fn test_list_traces() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_span(&create_test_span("span1", "trace1"))
            .unwrap();
        storage
            .insert_span(&create_test_span("span2", "trace2"))
            .unwrap();

        let traces = storage.list_traces(None, None).unwrap();
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn test_count_spans() {
        let storage = Storage::new_in_memory().unwrap();
        assert_eq!(storage.count_spans().unwrap(), 0);

        storage
            .insert_span(&create_test_span("span1", "trace1"))
            .unwrap();
        assert_eq!(storage.count_spans().unwrap(), 1);

        storage
            .insert_span(&create_test_span("span2", "trace1"))
            .unwrap();
        assert_eq!(storage.count_spans().unwrap(), 2);
    }

    #[test]
    fn test_insert_and_list_logs() {
        let storage = Storage::new_in_memory().unwrap();
        let log = Log::new(
            1_000_000_000,
            crate::models::SeverityLevel::Info,
            Some("INFO".to_string()),
            "Test log".to_string(),
            Attributes::new(),
            None,
            None,
            Some("test-service".to_string()),
        );

        storage.insert_log(&log).unwrap();
        let logs = storage.list_logs(None, None).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].body, "Test log");
    }

    #[test]
    fn test_get_nonexistent_trace() {
        let storage = Storage::new_in_memory().unwrap();
        let result = storage.get_trace_by_id("nonexistent");
        assert!(result.is_err());
    }
}
