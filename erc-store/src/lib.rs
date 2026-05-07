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
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT payload FROM receipts WHERE receipt_id = ?1",
            [receipt_id],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn get_events_by_execution_id(&self, execution_id: &str) -> Vec<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT payload FROM events WHERE execution_id = ?1 ORDER BY timestamp")
            .unwrap();
        let rows = stmt.query_map([execution_id], |row| row.get(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_events_by_trace_id(&self, trace_id: &str) -> Vec<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT payload FROM events WHERE trace_id = ?1 ORDER BY timestamp")
            .unwrap();
        let rows = stmt.query_map([trace_id], |row| row.get(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }
}