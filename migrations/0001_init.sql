-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Core: files
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    language TEXT,
    sloc INTEGER DEFAULT 0,
    last_modified TEXT
);

-- Core: symbols
CREATE TABLE IF NOT EXISTS symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    start_line INTEGER,
    end_line INTEGER,
    signature TEXT
);
CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);

-- Core: edges (knowledge graph)
CREATE TABLE IF NOT EXISTS edges (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES symbols(id),
    target_id INTEGER NOT NULL REFERENCES symbols(id),
    kind TEXT NOT NULL,
    confidence REAL DEFAULT 1.0
);
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);

-- Graph: communities
CREATE TABLE IF NOT EXISTS communities (
    id INTEGER PRIMARY KEY,
    name TEXT,
    description TEXT,
    cohesion_score REAL
);

CREATE TABLE IF NOT EXISTS community_members (
    community_id INTEGER NOT NULL REFERENCES communities(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    PRIMARY KEY (community_id, symbol_id)
);

-- Graph: processes (execution flows)
CREATE TABLE IF NOT EXISTS processes (
    id INTEGER PRIMARY KEY,
    name TEXT,
    entry_symbol_id INTEGER REFERENCES symbols(id),
    description TEXT
);

CREATE TABLE IF NOT EXISTS process_steps (
    process_id INTEGER NOT NULL REFERENCES processes(id),
    step_order INTEGER NOT NULL,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    PRIMARY KEY (process_id, step_order)
);

-- Metrics
CREATE TABLE IF NOT EXISTS file_metrics (
    file_id INTEGER PRIMARY KEY REFERENCES files(id),
    sloc INTEGER DEFAULT 0,
    methods INTEGER DEFAULT 0,
    fan_in INTEGER DEFAULT 0,
    fan_out INTEGER DEFAULT 0,
    complexity INTEGER DEFAULT 0,
    churn INTEGER DEFAULT 0,
    contributors INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS patterns (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    severity TEXT NOT NULL,
    file_id INTEGER REFERENCES files(id),
    symbol_id INTEGER REFERENCES symbols(id),
    description TEXT
);

-- Narratives (pre-generated LLM content)
CREATE TABLE IF NOT EXISTS narratives (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    target_id INTEGER,
    language TEXT DEFAULT 'en',
    content TEXT NOT NULL,
    generated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_narratives_kind ON narratives(kind, target_id);

-- Learning
CREATE TABLE IF NOT EXISTS chapters (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    order_index INTEGER NOT NULL,
    community_id INTEGER REFERENCES communities(id),
    difficulty TEXT DEFAULT 'beginner'
);

CREATE TABLE IF NOT EXISTS chapter_sections (
    id INTEGER PRIMARY KEY,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    title TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content TEXT,
    order_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS progress (
    id INTEGER PRIMARY KEY,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    section_id INTEGER REFERENCES chapter_sections(id),
    completed INTEGER DEFAULT 0,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS quiz_questions (
    id INTEGER PRIMARY KEY,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    kind TEXT DEFAULT 'multiple_choice'
);

CREATE TABLE IF NOT EXISTS quiz_attempts (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES quiz_questions(id),
    user_answer TEXT,
    correct INTEGER DEFAULT 0,
    attempted_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS badges (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    icon TEXT,
    earned_at TEXT
);

CREATE TABLE IF NOT EXISTS learner_stats (
    id INTEGER PRIMARY KEY,
    total_xp INTEGER DEFAULT 0,
    streak_days INTEGER DEFAULT 0,
    last_active TEXT
);

-- Harvest (trending repo tracking)
CREATE TABLE IF NOT EXISTS harvested_repos (
    id INTEGER PRIMARY KEY,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT,
    stars_today INTEGER DEFAULT 0,
    language TEXT,
    description TEXT,
    harvested_date TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    analyzed_at TEXT,
    exported_at TEXT,
    UNIQUE(owner, name, harvested_date)
);

-- Events (batch writer target)
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL,
    data TEXT,
    timestamp TEXT NOT NULL
);

-- Record schema version
INSERT INTO schema_version (version) VALUES (1);
