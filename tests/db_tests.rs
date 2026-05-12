use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn test_db_logic() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_db.db");
    let conn = Connection::open(db_path).unwrap();
    
    conn.execute(
        "CREATE TABLE codes (
            id INTEGER PRIMARY KEY,
            code TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();

    let stmt_insert = "INSERT INTO codes (code) VALUES (?)";
    conn.execute(stmt_insert, ["123456"]).unwrap();

    let mut stmt = conn.prepare("SELECT 1 FROM codes WHERE code = ? LIMIT 1").unwrap();
    let exists = stmt.exists(["123456"]).unwrap();
    assert!(exists, "Le code 123456 devrait exister");
    
    let exists_not = stmt.exists(["000000"]).unwrap();
    assert!(!exists_not, "Le code 000000 ne devrait pas exister");
}
