-- Performance indexes on foreign keys used in JOINs/WHEREs

CREATE INDEX IF NOT EXISTS idx_community_members_community ON community_members(community_id);
CREATE INDEX IF NOT EXISTS idx_community_members_symbol ON community_members(symbol_id);
CREATE INDEX IF NOT EXISTS idx_process_steps_process ON process_steps(process_id);
CREATE INDEX IF NOT EXISTS idx_chapters_community ON chapters(community_id);
CREATE INDEX IF NOT EXISTS idx_chapter_sections_chapter ON chapter_sections(chapter_id);
CREATE INDEX IF NOT EXISTS idx_progress_chapter ON progress(chapter_id);

INSERT INTO schema_version (version) VALUES (10);
