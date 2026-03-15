-- Add options, correct_index, explanation columns to quiz_questions.
-- SQLite ALTER TABLE only supports ADD COLUMN, so we add new columns.

ALTER TABLE quiz_questions ADD COLUMN options TEXT DEFAULT '[]';
ALTER TABLE quiz_questions ADD COLUMN correct_index INTEGER DEFAULT 0;
ALTER TABLE quiz_questions ADD COLUMN explanation TEXT DEFAULT '';
