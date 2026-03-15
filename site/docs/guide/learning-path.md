# Learning Path

The Learning Path is Codeilus's core feature &mdash; a structured, gamified journey through any codebase.

## How Chapters Are Generated

Codeilus doesn't guess at structure. It uses real graph data:

1. **Community detection** finds natural module boundaries (Louvain algorithm)
2. **Topological sort** orders modules by dependency (foundations first)
3. **Entry point scoring** identifies where to start reading
4. **Complexity metrics** set difficulty ratings

The result: a curriculum ordered so you always learn prerequisites before the things that depend on them.

## Chapter Structure

Each chapter contains:

- **Overview** &mdash; LLM-generated module explanation with beginner-friendly analogies
- **Key Concepts** &mdash; TF-IDF keywords with descriptions
- **Code Walkthrough** &mdash; guided reading order for the module's key files
- **Connections** &mdash; how this module relates to others
- **Quiz** &mdash; multiple choice, true/false, and impact analysis questions

## Special Chapters

- **Chapter 0: The Big Picture** &mdash; project overview, architecture diagram, entry points
- **Final Chapter: Putting It All Together** &mdash; cross-cutting flows and how everything connects

## Gamification

### XP System

| Action | XP |
|---|---|
| Complete a section | +10 |
| Complete a chapter | +50 |
| Pass a quiz | +25 |
| Explore graph | +5 |
| Ask a Q&A question | +5 |

### Badges

| Badge | Requirement |
|---|---|
| First Steps | Complete Chapter 0 |
| Chapter Champion | Complete any chapter |
| Graph Explorer | Visit 10 different nodes |
| Quiz Master | Pass 5 quizzes |
| Deep Diver | Read 20 symbol explanations |
| Completionist | 100% progress |

### Streaks

Consecutive days of activity are tracked. Your streak counter appears in the Learning Path header.

## Progress Tracking

Progress is tracked per-section and per-chapter. The Learning Path page shows:

- Overall completion percentage
- Per-chapter progress bars
- XP counter and streak days
- Earned badges
