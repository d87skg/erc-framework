@"
use rusqlite::Connection;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(db_path: &str) -> Self {
        let conn = Connection::open(db_path).expect("无法打开数据库");
        Self { conn }
    }

    pub fn get_receipt(&self, receipt_id: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT payload FROM receipts WHERE receipt_id = ?1",
                [receipt_id],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn get_events_by_execution_id(&self, execution_id: &str) -> Vec<String> {
        let mut stmt = self.conn
            .prepare("SELECT payload FROM events WHERE execution_id = ?1 ORDER BY timestamp")
            .unwrap();
        let rows = stmt.query_map([execution_id], |row| row.get(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_events_by_trace_id(&self, trace_id: &str) -> Vec<String> {
        let mut stmt = self.conn
            .prepare("SELECT payload FROM events WHERE trace_id = ?1 ORDER BY timestamp")
            .unwrap();
        let rows = stmt.query_map([trace_id], |row| row.get(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }
}
"@ | Set-Content -Path "D:\erc-project\erc-store\src\lib.rs" -Encoding UTF8