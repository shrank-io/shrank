use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )?;

    if current < 1 {
        migrate_v1(conn)?;
        conn.execute("INSERT INTO _migrations (version) VALUES (1)", [])?;
        tracing::info!("applied migration v1: initial schema");
    }

    if current < 2 {
        migrate_v2(conn)?;
        conn.execute("INSERT INTO _migrations (version) VALUES (2)", [])?;
        tracing::info!("applied migration v2: add ocr_markdown column");
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        -- Documents: the primary entity
        CREATE TABLE documents (
            id              TEXT PRIMARY KEY,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            captured_at     TEXT NOT NULL,
            synced_at       TEXT,

            original_path   TEXT NOT NULL,
            thumbnail_path  TEXT NOT NULL,

            status          TEXT NOT NULL DEFAULT 'pending',
            processing_error TEXT,
            raw_llm_response TEXT,

            language        TEXT,
            sender          TEXT,
            sender_normalized TEXT,
            document_date   TEXT,
            document_type   TEXT,
            subject         TEXT,
            extracted_text  TEXT,

            amounts         TEXT,
            dates           TEXT,
            reference_ids   TEXT,

            tags            TEXT,
            confidence      REAL
        );

        -- Full-text search index
        CREATE VIRTUAL TABLE documents_fts USING fts5(
            sender,
            subject,
            extracted_text,
            tags,
            content='documents',
            content_rowid='rowid',
            tokenize='unicode61 remove_diacritics 2'
        );

        -- Triggers to keep FTS in sync
        CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
            INSERT INTO documents_fts(rowid, sender, subject, extracted_text, tags)
            VALUES (new.rowid, new.sender, new.subject, new.extracted_text, new.tags);
        END;

        CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extracted_text, tags)
            VALUES ('delete', old.rowid, old.sender, old.subject, old.extracted_text, old.tags);
        END;

        CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extracted_text, tags)
            VALUES ('delete', old.rowid, old.sender, old.subject, old.extracted_text, old.tags);
            INSERT INTO documents_fts(rowid, sender, subject, extracted_text, tags)
            VALUES (new.rowid, new.sender, new.subject, new.extracted_text, new.tags);
        END;

        -- Entities: things that appear across multiple documents
        CREATE TABLE entities (
            id              TEXT PRIMARY KEY,
            entity_type     TEXT NOT NULL,
            value           TEXT NOT NULL,
            display_name    TEXT,
            metadata        TEXT,
            created_at      TEXT NOT NULL,
            UNIQUE(entity_type, value)
        );

        -- Links between documents and entities
        CREATE TABLE document_entities (
            document_id     TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            entity_id       TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
            role            TEXT NOT NULL,
            confidence      REAL DEFAULT 1.0,
            PRIMARY KEY (document_id, entity_id, role)
        );

        -- Direct document-to-document relationships
        CREATE TABLE document_edges (
            source_id       TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            target_id       TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            relation_type   TEXT NOT NULL,
            confidence      REAL DEFAULT 1.0,
            inferred_by     TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            PRIMARY KEY (source_id, target_id, relation_type)
        );

        -- Indexes for graph traversal
        CREATE INDEX idx_doc_entities_entity ON document_entities(entity_id);
        CREATE INDEX idx_doc_entities_doc ON document_entities(document_id);
        CREATE INDEX idx_doc_edges_source ON document_edges(source_id);
        CREATE INDEX idx_doc_edges_target ON document_edges(target_id);

        -- Sync state
        CREATE TABLE sync_cursors (
            client_id       TEXT PRIMARY KEY,
            last_sync_at    TEXT NOT NULL,
            last_document_id TEXT
        );

        -- Additional indexes for common queries
        CREATE INDEX idx_documents_status ON documents(status);
        CREATE INDEX idx_documents_sender ON documents(sender_normalized);
        CREATE INDEX idx_documents_type ON documents(document_type);
        CREATE INDEX idx_documents_date ON documents(document_date);
        CREATE INDEX idx_documents_updated ON documents(updated_at);
        ",
    )?;

    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "ALTER TABLE documents ADD COLUMN ocr_markdown TEXT;",
    )?;
    Ok(())
}
