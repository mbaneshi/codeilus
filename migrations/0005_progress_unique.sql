-- Add unique constraint on progress(chapter_id, section_id) to support upserts.
CREATE UNIQUE INDEX IF NOT EXISTS idx_progress_chapter_section
ON progress(chapter_id, section_id);
