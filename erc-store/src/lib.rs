use rusqlite::Connection;
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_receipt(&self, receipt_id: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap_or_else(|e| {
            tracing::error!("数据库锁被污染: {}", e);
            std::process::exit(1);
        });
        conn.query_row(
            "SELECT payload FROM receipts WHERE receipt_id = ?1",
            [receipt_id],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn get_events_by_execution_id(&self, execution_id: &str) -> Vec<String> {
        let conn = self.conn.lock().unwrap_or_else(|e| {
            tracing::error!("数据库锁被污染: {}", e);
            std::process::exit(1);
        });
        let mut stmt = match conn
            .prepare("SELECT payload FROM events WHERE execution_id = ?1 ORDER BY timestamp")
        {
            Ok(stmt) => stmt,
            Err(e) => {
                tracing::error!("准备SQL语句失败: {}", e);
                return vec![];
            }
        };
        let rows = match stmt.query_map([execution_id], |row| row.get(0)) {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("查询事件失败: {}", e);
                return vec![];
            }
        };
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_events_by_trace_id(&self, trace_id: &str) -> Vec<String> {
        let conn = self.conn.lock().unwrap_or_else(|e| {
            tracing::error!("数据库锁被污染: {}", e);
            std::process::exit(1);
        });
        let mut stmt = match conn
            .prepare("SELECT payload FROM events WHERE trace_id = ?1 ORDER BY timestamp")
        {
            Ok(stmt) => stmt,
            Err(e) => {
                tracing::error!("准备SQL语句失败: {}", e);
                return vec![];
            }
        };
        let rows = match stmt.query_map([trace_id], |row| row.get(0)) {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("查询事件失败: {}", e);
                return vec![];
            }
        };
        rows.filter_map(|r| r.ok()).collect()
    }
}