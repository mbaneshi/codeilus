-- FTS5 virtual tables for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    path, language,
    content=files, content_rowid=id
);

CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
    name, kind, signature,
    content=symbols, content_rowid=id
);

CREATE VIRTUAL TABLE IF NOT EXISTS narratives_fts USING fts5(
    kind, content,
    content=narratives, content_rowid=id
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, path, language) VALUES (new.id, new.path, new.language);
END;
CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, path, language) VALUES('delete', old.id, old.path, old.language);
END;
CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, path, language) VALUES('delete', old.id, old.path, old.language);
    INSERT INTO files_fts(rowid, path, language) VALUES (new.id, new.path, new.language);
END;

CREATE TRIGGER IF NOT EXISTS symbols_ai AFTER INSERT ON symbols BEGIN
    INSERT INTO symbols_fts(rowid, name, kind, signature) VALUES (new.id, new.name, new.kind, new.signature);
END;
CREATE TRIGGER IF NOT EXISTS symbols_ad AFTER DELETE ON symbols BEGIN
    INSERT INTO symbols_fts(symbols_fts, rowid, name, kind, signature) VALUES('delete', old.id, old.name, old.kind, old.signature);
END;
CREATE TRIGGER IF NOT EXISTS symbols_au AFTER UPDATE ON symbols BEGIN
    INSERT INTO symbols_fts(symbols_fts, rowid, name, kind, signature) VALUES('delete', old.id, old.name, old.kind, old.signature);
    INSERT INTO symbols_fts(rowid, name, kind, signature) VALUES (new.id, new.name, new.kind, new.signature);
END;

CREATE TRIGGER IF NOT EXISTS narratives_ai AFTER INSERT ON narratives BEGIN
    INSERT INTO narratives_fts(rowid, kind, content) VALUES (new.id, new.kind, new.content);
END;
CREATE TRIGGER IF NOT EXISTS narratives_ad AFTER DELETE ON narratives BEGIN
    INSERT INTO narratives_fts(narratives_fts, rowid, kind, content) VALUES('delete', old.id, old.kind, old.content);
END;
CREATE TRIGGER IF NOT EXISTS narratives_au AFTER UPDATE ON narratives BEGIN
    INSERT INTO narratives_fts(narratives_fts, rowid, kind, content) VALUES('delete', old.id, old.kind, old.content);
    INSERT INTO narratives_fts(rowid, kind, content) VALUES (new.id, new.kind, new.content);
END;

-- Populate FTS from existing data
INSERT INTO files_fts(rowid, path, language) SELECT id, path, language FROM files;
INSERT INTO symbols_fts(rowid, name, kind, signature) SELECT id, name, kind, signature FROM symbols;
INSERT INTO narratives_fts(rowid, kind, content) SELECT id, kind, content FROM narratives;

INSERT INTO schema_version (version) VALUES (2);
