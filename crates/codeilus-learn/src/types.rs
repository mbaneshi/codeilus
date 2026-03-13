//! Learning engine types: curriculum, chapters, progress, gamification, quizzes.

use codeilus_core::ids::{ChapterId, CommunityId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    pub chapters: Vec<Chapter>,
    pub total_sections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub order: usize,
    pub title: String,
    pub description: String,
    pub community_id: Option<CommunityId>,
    pub sections: Vec<Section>,
    pub difficulty: Difficulty,
    pub prerequisite_ids: Vec<ChapterId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionKind {
    Overview,
    KeyConcepts,
    Diagram,
    CodeWalkthrough,
    Connections,
    Quiz,
}

impl SectionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::KeyConcepts => "key_concepts",
            Self::Diagram => "diagram",
            Self::CodeWalkthrough => "code_walkthrough",
            Self::Connections => "connections",
            Self::Quiz => "quiz",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::KeyConcepts => "Key Concepts",
            Self::Diagram => "Diagram",
            Self::CodeWalkthrough => "Code Walkthrough",
            Self::Connections => "Connections",
            Self::Quiz => "Quiz",
        }
    }

    pub fn all() -> &'static [SectionKind] {
        &[
            Self::Overview,
            Self::KeyConcepts,
            Self::Diagram,
            Self::CodeWalkthrough,
            Self::Connections,
            Self::Quiz,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Advanced => "advanced",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub chapter_id: ChapterId,
    pub section_id: String,
    pub xp_earned: i64,
    pub badges_earned: Vec<Badge>,
    pub total_xp: i64,
    pub overall_progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerStats {
    pub total_xp: i64,
    pub level: usize,
    pub badges: Vec<Badge>,
    pub streak_days: usize,
    pub chapters_completed: usize,
    pub sections_completed: usize,
    pub quizzes_passed: usize,
    pub overall_progress: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Badge {
    FirstSteps,
    ChapterChampion,
    GraphExplorer,
    QuizMaster,
    DeepDiver,
    Completionist,
    Polyglot,
    CodeDetective,
}

impl Badge {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FirstSteps => "first_steps",
            Self::ChapterChampion => "chapter_champion",
            Self::GraphExplorer => "graph_explorer",
            Self::QuizMaster => "quiz_master",
            Self::DeepDiver => "deep_diver",
            Self::Completionist => "completionist",
            Self::Polyglot => "polyglot",
            Self::CodeDetective => "code_detective",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "first_steps" => Some(Self::FirstSteps),
            "chapter_champion" => Some(Self::ChapterChampion),
            "graph_explorer" => Some(Self::GraphExplorer),
            "quiz_master" => Some(Self::QuizMaster),
            "deep_diver" => Some(Self::DeepDiver),
            "completionist" => Some(Self::Completionist),
            "polyglot" => Some(Self::Polyglot),
            "code_detective" => Some(Self::CodeDetective),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::FirstSteps => "Complete Chapter 0: The Big Picture",
            Self::ChapterChampion => "Complete any chapter",
            Self::GraphExplorer => "Visit 10 different graph nodes",
            Self::QuizMaster => "Pass 5 quizzes",
            Self::DeepDiver => "Read 20 symbol explanations",
            Self::Completionist => "100% progress",
            Self::Polyglot => "Explore files in 3+ languages",
            Self::CodeDetective => "Find 3 anti-patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub chapter_id: ChapterId,
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: String,
    pub question: String,
    pub kind: QuizQuestionKind,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuizQuestionKind {
    MultipleChoice,
    TrueFalse,
    ImpactAnalysis,
}
