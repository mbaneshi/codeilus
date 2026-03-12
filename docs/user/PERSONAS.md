# Codeilus — User Personas & Journeys

> Who uses Codeilus, what they feel, what they need, and how they succeed.

---

## Persona 1: Nadia — The New Hire

**Role:** Junior-to-mid backend developer, 2 years experience
**Context:** Just joined a team with a 200K-line Python monolith. No onboarding docs.
**Emotional state:** Overwhelmed, impostor syndrome, afraid to ask "dumb" questions

### Pain Points
- Opens random files, can't tell what's important vs. boilerplate
- Grep returns 500 results — which ones matter?
- Asks teammates but feels like a burden
- README is 3 years old and wrong
- After 2 weeks, still can't explain what the app does to someone else

### What Success Looks Like
- Day 1: Runs `codeilus ./repo` → reads Chapter 0 "Big Picture" → can explain the app's architecture in 2 sentences
- Day 2: Completes Chapter 1 (core module) → understands the data model
- Day 3: Asks Q&A "how does auth work?" → gets answer with exact file references
- Week 1: 40% progress, earned 3 badges, can navigate confidently
- Week 2: Submits first PR with confidence, knows blast radius of her change

### Key Codeilus Features Used
1. Learning path (chapters in order)
2. Q&A ("explain this function to me like I'm new")
3. Architecture diagram (mental model in 30 seconds)
4. Blast radius (before submitting PR)

### UX Requirements
- Language must be beginner-friendly — no jargon without explanation
- Progress must feel achievable — small sections, frequent XP rewards
- Never make the user feel dumb — explanations use analogies
- "Ask" must feel safe — no judgment, patient responses

---

## Persona 2: Marcus — The Senior Engineer

**Role:** Senior backend + infra, 10 years experience
**Context:** Inherited a legacy Go microservices codebase (15 services, 80K lines). Previous team left.
**Emotional state:** Frustrated, time-pressured, needs answers fast

### Pain Points
- No one alive knows why certain services exist
- Circular dependencies between services he can't untangle
- Needs to deprecate 3 services but doesn't know what breaks
- Documentation exists but contradicts the code
- Spends 60% of time reading, 40% coding

### What Success Looks Like
- Minute 1: `codeilus ./monorepo` → architecture diagram shows all 15 services with connections
- Minute 5: Metrics heatmap shows the 3 most complex services
- Minute 10: Anti-pattern detection flags circular dependencies he suspected
- Minute 30: Blast radius analysis confirms he can safely remove `legacy-auth-proxy`
- Hour 1: Exports static page for the team: "here's our architecture, here are the problems"

### Key Codeilus Features Used
1. Architecture diagram (immediate mental model)
2. Metrics heatmap (where are the problems?)
3. Anti-pattern detection (circular deps, god classes)
4. Blast radius / what-if (safe to remove?)
5. Export (share findings with team)

### UX Requirements
- Speed over hand-holding — show data, not tutorials
- Metrics must be trustworthy — wrong data is worse than no data
- Export must be shareable — team needs to see this without installing anything
- Graph must handle 500+ nodes without lag

---

## Persona 3: Sofia — The Open Source Contributor

**Role:** Full-stack developer, contributes to OSS on weekends
**Context:** Found an interesting library on GitHub Trending. Wants to contribute but doesn't know the codebase.
**Emotional state:** Curious, motivated, but won't spend more than 30 minutes deciding

### Pain Points
- README says "contributions welcome" but doesn't say where to start
- Codebase has 200 files — which 5 should she read?
- Doesn't understand the module boundaries
- Submitted a PR once that broke something she didn't know existed

### What Success Looks Like
- Sees the repo on codeilus.dev daily digest → clicks → reads 30-second overview
- Architecture diagram shows 4 main modules → she understands the shape
- "Key files to read" list → reads 3 files → understands 80%
- "How to extend" guide → knows exactly where to add her feature
- Contributes successfully, becomes a regular contributor

### Key Codeilus Features Used
1. Static grasp page (30-second overview without cloning)
2. Reading order (which 3-5 files to read)
3. Extension guide (how to add features)
4. Entry points (where does execution start?)

### UX Requirements
- Must work without cloning — static page or web view
- 30-second overview must be genuinely useful (not marketing fluff)
- Reading order must be correct — wrong order wastes time and trust
- Mobile-friendly — she browses GitHub Trending on her phone

---

## Persona 4: David — The Tech Lead

**Role:** Engineering manager / tech lead, 8 years experience
**Context:** Responsible for a team of 6, maintaining a TypeScript monorepo (150K lines)
**Emotional state:** Strategic, time-poor, needs to make decisions with data

### Pain Points
- Team velocity is dropping but he can't pinpoint why
- Suspects technical debt but can't quantify it
- New features take longer because "everything is connected"
- Code reviews take too long because reviewers don't understand the blast radius
- Onboarding new hires takes 3-4 weeks

### What Success Looks Like
- Runs Codeilus quarterly → tracks complexity trends over time
- Metrics dashboard shows: "auth module complexity increased 40% this quarter"
- Anti-pattern report shows 3 god classes and 2 circular dependency cycles
- Shares learning path with new hires → onboarding drops to 1 week
- Before big refactors: blast radius analysis justifies the effort to stakeholders

### Key Codeilus Features Used
1. Metrics dashboard (complexity trends, hotspots)
2. Anti-pattern detection (quantified tech debt)
3. Learning path (for his team's onboarding)
4. Blast radius (justify refactoring decisions)
5. Export (share with stakeholders)

### UX Requirements
- Metrics must be exportable (screenshots, reports)
- Trends over time (compare this quarter vs. last)
- Language appropriate for non-technical stakeholders in exports
- MCP integration so his team's AI tools understand the codebase

---

## Persona 5: Professor Chen — The Educator

**Role:** CS professor, teaches Software Engineering course
**Context:** Uses real open-source projects as teaching examples. Currently spends hours preparing materials manually.
**Emotional state:** Passionate about teaching, frustrated by prep time

### Pain Points
- Spends 4 hours per project preparing "guided tour" slides
- Students can't navigate real codebases on their own
- No way to verify students actually explored the code
- Examples go stale when the repo updates

### What Success Looks Like
- Points `codeilus` at any repo → gets a complete curriculum in minutes
- Students follow the learning path independently
- Quizzes verify understanding without manual grading
- Progress tracking shows which students are stuck
- Can re-run on updated repo → curriculum regenerates

### Key Codeilus Features Used
1. Learning path (auto-generated curriculum)
2. Quizzes (auto-generated from graph data)
3. Progress tracking (see student completion)
4. Architecture diagrams (visual aids for lectures)

### UX Requirements
- Curriculum must be pedagogically sound (foundational → advanced)
- Quizzes must be fair — answerable from the chapter content
- Progress must persist across sessions
- Works offline (classroom may have poor internet)

---

## User Journey Map

### Journey 1: "Learn a Codebase" (Nadia, Professor Chen)

```
Install        Analyze         Orient           Learn            Master
───────── → ────────── → ──────────── → ────────────── → ──────────────
cargo         codeilus       Welcome page      Chapter 1→N       100% complete
install       ./repo         Architecture      Guided reading    All badges
codeilus                     diagram           Quizzes           Blast radius
                             "Big Picture"     XP + badges       Export & share
                             chapter 0         Progress bar

Emotion:  Hopeful → Impressed → Oriented → Engaged → Confident
Time:     1 min     30 sec      5 min       hours     ongoing
```

### Journey 2: "Assess a Codebase" (Marcus, David)

```
Install        Analyze         Scan              Investigate       Act
───────── → ────────── → ──────────── → ────────────── → ──────────────
cargo         codeilus       Architecture      Metrics heatmap    Blast radius
install       ./repo         diagram           Anti-patterns      What-if removal
codeilus                     Entry points      God classes        Export report
                                               Circular deps      Share with team

Emotion:  Skeptical → Curious → "Aha!" → Data-driven → Empowered
Time:     1 min       30 sec    2 min     10 min        30 min
```

### Journey 3: "Quick Grasp" (Sofia, browsing)

```
Discover       Click           Scan              Decide           Contribute
───────── → ────────── → ──────────── → ────────────── → ──────────────
GitHub        codeilus.dev    30-sec overview   Key files to      Clone + extend
Trending      /repo-page     Architecture      read              guided by
                             diagram           Extension guide    Codeilus

Emotion:  Curious → Interested → "I get it" → "I can do this" → Contributing
Time:     0          5 sec       30 sec        5 min              ongoing
```

---

## Key UX Principles (derived from personas)

1. **Speed to insight** — first useful information in < 60 seconds
2. **Progressive disclosure** — overview first, details on demand
3. **Never condescend** — explain clearly, but respect intelligence
4. **Data over opinion** — metrics, not vibes
5. **Shareable outputs** — everything exportable for teams
6. **Graceful degradation** — works without LLM, better with it
7. **Mobile-aware** — static pages must work on phones
8. **Offline-capable** — classroom and airplane mode
9. **Trust through accuracy** — wrong data destroys credibility permanently
10. **Celebrate progress** — XP, badges, streaks make learning feel good
