use duckdb::{Connection, Result};

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute("CREATE SEQUENCE IF NOT EXISTS log_id_seq START 1", [])?;
    conn.execute("CREATE SEQUENCE IF NOT EXISTS metric_id_seq START 1", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS spans (
            span_id TEXT NOT NULL,
            trace_id TEXT NOT NULL,
            parent_span_id TEXT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            start_time_unix_nano BIGINT NOT NULL,
            end_time_unix_nano BIGINT NOT NULL,
            attributes TEXT NOT NULL,
            status TEXT NOT NULL,
            service_name TEXT,
            PRIMARY KEY (span_id, trace_id)
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_spans_trace_id ON spans(trace_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_spans_service_name ON spans(service_name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_spans_start_time ON spans(start_time_unix_nano)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY DEFAULT nextval('log_id_seq'),
            time_unix_nano BIGINT NOT NULL,
            severity_level TEXT NOT NULL,
            severity_text TEXT,
            body TEXT NOT NULL,
            attributes TEXT NOT NULL,
            trace_id TEXT,
            span_id TEXT,
            service_name TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_time ON logs(time_unix_nano)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_trace_id ON logs(trace_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_service_name ON logs(service_name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_severity ON logs(severity_level)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY DEFAULT nextval('metric_id_seq'),
            name TEXT NOT NULL,
            description TEXT,
            unit TEXT,
            metric_type TEXT NOT NULL,
            temporality TEXT NOT NULL,
            time_unix_nano BIGINT NOT NULL,
            start_time_unix_nano BIGINT,
            value DOUBLE NOT NULL,
            attributes TEXT NOT NULL,
            service_name TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_metrics_time ON metrics(time_unix_nano)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_metrics_service_name ON metrics(service_name)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_schema() {
        let conn = Connection::open_in_memory().unwrap();
        let result = init_schema(&conn);
        assert!(result.is_ok());

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"spans".to_string()));
        assert!(tables.contains(&"logs".to_string()));
        assert!(tables.contains(&"metrics".to_string()));
    }

    #[test]
    fn test_init_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_schema(&conn).is_ok());
        assert!(init_schema(&conn).is_ok());
        assert!(init_schema(&conn).is_ok());
    }
}
