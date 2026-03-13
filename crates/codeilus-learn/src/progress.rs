//! Progress tracking with XP, badges, and streaks.

use codeilus_core::ids::{ChapterId, SymbolId};
use codeilus_core::CodeilusResult;
use codeilus_db::{ChapterRepo, ProgressRepo};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tracing::debug;

use crate::types::{Badge, LearnerStats, ProgressUpdate};

/// XP awards per action.
const XP_SECTION: i64 = 10;
const XP_CHAPTER_BONUS: i64 = 50;
const XP_QUIZ: i64 = 25;
const XP_EXPLORE: i64 = 5;
const XP_QUESTION: i64 = 5;

pub struct ProgressTracker {
    progress_repo: ProgressRepo,
    chapter_repo: ChapterRepo,
}

impl ProgressTracker {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            progress_repo: ProgressRepo::new(Arc::clone(&conn)),
            chapter_repo: ChapterRepo::new(conn),
        }
    }

    /// Record completion of a section. Returns XP earned and any new badges.
    pub fn complete_section(
        &self,
        chapter_id: ChapterId,
        section_id: &str,
    ) -> CodeilusResult<ProgressUpdate> {
        // Record the section completion
        self.progress_repo
            .record_section(chapter_id.0, section_id)?;

        let mut xp_earned = XP_SECTION;

        // Check if the chapter is now complete for bonus XP
        let progress = self.progress_repo.get_chapter_progress(chapter_id.0)?;
        if (progress - 1.0).abs() < f64::EPSILON {
            xp_earned += XP_CHAPTER_BONUS;
        }

        // Update XP
        let mut stats = self.progress_repo.get_or_create_stats()?;
        stats.total_xp += xp_earned;
        stats.last_active = chrono::Utc::now().to_rfc3339();
        self.progress_repo.update_stats(&stats)?;

        // Check for new badges
        let badges_earned = self.check_badges()?;
        let overall_progress = self.progress_repo.get_overall_progress()?;

        debug!(
            chapter = chapter_id.0,
            section = section_id,
            xp_earned,
            "recorded section completion"
        );

        Ok(ProgressUpdate {
            chapter_id,
            section_id: section_id.to_string(),
            xp_earned,
            badges_earned,
            total_xp: stats.total_xp,
            overall_progress,
        })
    }

    /// Record a quiz attempt.
    pub fn record_quiz(
        &self,
        chapter_id: ChapterId,
        score: f64,
        passed: bool,
    ) -> CodeilusResult<ProgressUpdate> {
        self.progress_repo
            .record_quiz_attempt(chapter_id.0, score, passed)?;

        let mut xp_earned = 0;
        if passed {
            xp_earned = XP_QUIZ;
        }

        let mut stats = self.progress_repo.get_or_create_stats()?;
        stats.total_xp += xp_earned;
        stats.last_active = chrono::Utc::now().to_rfc3339();
        self.progress_repo.update_stats(&stats)?;

        let badges_earned = self.check_badges()?;
        let overall_progress = self.progress_repo.get_overall_progress()?;

        Ok(ProgressUpdate {
            chapter_id,
            section_id: "quiz".to_string(),
            xp_earned,
            badges_earned,
            total_xp: stats.total_xp,
            overall_progress,
        })
    }

    /// Record graph exploration (viewing a node).
    pub fn record_explore(&self, _symbol_id: SymbolId) -> CodeilusResult<ProgressUpdate> {
        let mut stats = self.progress_repo.get_or_create_stats()?;
        stats.total_xp += XP_EXPLORE;
        stats.last_active = chrono::Utc::now().to_rfc3339();
        self.progress_repo.update_stats(&stats)?;

        let badges_earned = self.check_badges()?;
        let overall_progress = self.progress_repo.get_overall_progress()?;

        Ok(ProgressUpdate {
            chapter_id: ChapterId(0),
            section_id: "explore".to_string(),
            xp_earned: XP_EXPLORE,
            badges_earned,
            total_xp: stats.total_xp,
            overall_progress,
        })
    }

    /// Record asking a Q&A question.
    pub fn record_question(&self) -> CodeilusResult<ProgressUpdate> {
        let mut stats = self.progress_repo.get_or_create_stats()?;
        stats.total_xp += XP_QUESTION;
        stats.last_active = chrono::Utc::now().to_rfc3339();
        self.progress_repo.update_stats(&stats)?;

        let overall_progress = self.progress_repo.get_overall_progress()?;

        Ok(ProgressUpdate {
            chapter_id: ChapterId(0),
            section_id: "question".to_string(),
            xp_earned: XP_QUESTION,
            badges_earned: vec![],
            total_xp: stats.total_xp,
            overall_progress,
        })
    }

    /// Get current learner stats.
    pub fn get_stats(&self) -> CodeilusResult<LearnerStats> {
        let stats = self.progress_repo.get_or_create_stats()?;
        let badge_names = self.progress_repo.list_badges()?;
        let badges: Vec<Badge> = badge_names
            .iter()
            .filter_map(|name| Badge::parse(name))
            .collect();
        let chapters_completed = self.progress_repo.count_completed_chapters()?;
        let sections_completed = self.progress_repo.count_completed_sections()?;
        let quizzes_passed = self.progress_repo.count_quizzes_passed()?;
        let overall_progress = self.progress_repo.get_overall_progress()?;

        let level = compute_level(stats.total_xp);

        Ok(LearnerStats {
            total_xp: stats.total_xp,
            level,
            badges,
            streak_days: stats.streak_days as usize,
            chapters_completed,
            sections_completed,
            quizzes_passed,
            overall_progress,
        })
    }

    /// Get progress for a specific chapter.
    pub fn get_chapter_progress(&self, chapter_id: ChapterId) -> CodeilusResult<f64> {
        self.progress_repo.get_chapter_progress(chapter_id.0)
    }

    /// Check and award any newly earned badges.
    fn check_badges(&self) -> CodeilusResult<Vec<Badge>> {
        let mut new_badges = Vec::new();

        // FirstSteps: Chapter 0 (order_index=0) completed
        let chapters = self.chapter_repo.list_ordered()?;
        if let Some(ch0) = chapters.iter().find(|c| c.order_index == 0) {
            if self.progress_repo.is_chapter_complete(ch0.id.0)?
                && self
                    .progress_repo
                    .insert_badge(Badge::FirstSteps.as_str(), Badge::FirstSteps.description())?
            {
                new_badges.push(Badge::FirstSteps);
            }
        }

        // ChapterChampion: Any chapter complete
        let completed = self.progress_repo.count_completed_chapters()?;
        if completed > 0
            && self.progress_repo.insert_badge(
                Badge::ChapterChampion.as_str(),
                Badge::ChapterChampion.description(),
            )?
        {
            new_badges.push(Badge::ChapterChampion);
        }

        // QuizMaster: 5+ quizzes passed
        let quizzes = self.progress_repo.count_quizzes_passed()?;
        if quizzes >= 5
            && self
                .progress_repo
                .insert_badge(Badge::QuizMaster.as_str(), Badge::QuizMaster.description())?
        {
            new_badges.push(Badge::QuizMaster);
        }

        // Completionist: 100% progress
        let overall = self.progress_repo.get_overall_progress()?;
        if (overall - 1.0).abs() < f64::EPSILON
            && self.progress_repo.insert_badge(
                Badge::Completionist.as_str(),
                Badge::Completionist.description(),
            )?
        {
            new_badges.push(Badge::Completionist);
        }

        Ok(new_badges)
    }
}

fn compute_level(total_xp: i64) -> usize {
    // Level up every 100 XP
    (total_xp / 100) as usize + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_db::{DbPool, Migrator};

    fn setup_db() -> Arc<Mutex<Connection>> {
        let pool = DbPool::in_memory().unwrap();
        {
            let conn = pool.connection();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        pool.conn_arc()
    }

    fn setup_chapter_with_sections(conn: &Arc<Mutex<Connection>>, order: i64) -> ChapterId {
        let repo = ChapterRepo::new(Arc::clone(conn));
        let id = repo
            .insert(order, &format!("Chapter {}", order), "desc", None, "beginner")
            .unwrap();
        for kind in crate::types::SectionKind::all() {
            repo.insert_section(id, kind.as_str(), kind.title(), kind.as_str())
                .unwrap();
        }
        id
    }

    #[test]
    fn progress_section_xp() {
        let conn = setup_db();
        let ch_id = setup_chapter_with_sections(&conn, 0);
        let tracker = ProgressTracker::new(conn);
        let update = tracker.complete_section(ch_id, "overview").unwrap();
        assert_eq!(update.xp_earned, 10);
    }

    #[test]
    fn progress_chapter_bonus() {
        let conn = setup_db();
        let ch_id = setup_chapter_with_sections(&conn, 0);
        let tracker = ProgressTracker::new(conn);

        // Complete all 6 sections
        for kind in crate::types::SectionKind::all() {
            tracker.complete_section(ch_id, kind.as_str()).unwrap();
        }

        // Get stats to verify total XP
        let stats = tracker.get_stats().unwrap();
        // 6 sections × 10 XP + 50 bonus = 110
        // The bonus is added when the last section completes
        assert_eq!(stats.total_xp, 6 * 10 + 50);
    }

    #[test]
    fn progress_quiz_xp() {
        let conn = setup_db();
        let ch_id = setup_chapter_with_sections(&conn, 0);
        let tracker = ProgressTracker::new(conn);
        let update = tracker.record_quiz(ch_id, 1.0, true).unwrap();
        assert_eq!(update.xp_earned, 25);
    }

    #[test]
    fn progress_explore_xp() {
        let conn = setup_db();
        let _ch_id = setup_chapter_with_sections(&conn, 0);
        let tracker = ProgressTracker::new(conn);
        let update = tracker.record_explore(SymbolId(1)).unwrap();
        assert_eq!(update.xp_earned, 5);
    }

    #[test]
    fn badge_first_steps() {
        let conn = setup_db();
        let ch0 = setup_chapter_with_sections(&conn, 0);
        let tracker = ProgressTracker::new(conn);

        // Complete all sections of chapter 0
        for kind in crate::types::SectionKind::all() {
            tracker.complete_section(ch0, kind.as_str()).unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert!(
            stats.badges.contains(&Badge::FirstSteps),
            "Should have FirstSteps badge"
        );
    }

    #[test]
    fn badge_chapter_champion() {
        let conn = setup_db();
        let ch = setup_chapter_with_sections(&conn, 1); // non-zero chapter
        let tracker = ProgressTracker::new(conn);

        for kind in crate::types::SectionKind::all() {
            tracker.complete_section(ch, kind.as_str()).unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert!(
            stats.badges.contains(&Badge::ChapterChampion),
            "Should have ChapterChampion badge"
        );
    }

    #[test]
    fn badge_quiz_master() {
        let conn = setup_db();
        // Create 5 chapters with quiz sections
        let mut chapter_ids = Vec::new();
        for i in 0..5 {
            chapter_ids.push(setup_chapter_with_sections(&conn, i));
        }
        let tracker = ProgressTracker::new(conn);

        // Pass 5 quizzes
        for &ch_id in &chapter_ids {
            tracker.record_quiz(ch_id, 1.0, true).unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert!(
            stats.badges.contains(&Badge::QuizMaster),
            "Should have QuizMaster badge after 5 quizzes"
        );
    }

    #[test]
    fn badge_not_duplicated() {
        let conn = setup_db();
        let ch = setup_chapter_with_sections(&conn, 1);
        let tracker = ProgressTracker::new(conn);

        // Complete chapter twice (second time is a no-op for badges)
        for kind in crate::types::SectionKind::all() {
            tracker.complete_section(ch, kind.as_str()).unwrap();
        }

        let stats1 = tracker.get_stats().unwrap();
        let champion_count = stats1
            .badges
            .iter()
            .filter(|b| **b == Badge::ChapterChampion)
            .count();
        assert_eq!(champion_count, 1, "Badge should appear exactly once");

        // Try to trigger badge check again
        let _ = tracker.record_explore(SymbolId(1));
        let stats2 = tracker.get_stats().unwrap();
        let champion_count2 = stats2
            .badges
            .iter()
            .filter(|b| **b == Badge::ChapterChampion)
            .count();
        assert_eq!(champion_count2, 1, "Badge should still appear exactly once");
    }
}
