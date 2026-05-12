use rusqlite::{Connection, Result, params};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn init() -> Result<Self> {
        let path = "codes.db";
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS codes (
                id INTEGER PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Index pour la performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_codes_code ON codes (code)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn check_code(&self, code: &str) -> Result<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM codes WHERE code = ? LIMIT 1")?;
        let exists = stmt.exists(params![code])?;
        Ok(exists)
    }

    pub fn add_code(&self, code: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO codes (code) VALUES (?)",
            params![code],
        )?;
        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        self.conn.execute("DELETE FROM codes", [])?;
        Ok(())
    }
}
