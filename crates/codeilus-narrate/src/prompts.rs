use codeilus_core::types::NarrativeKind;

pub struct PromptTemplate {
    pub system: &'static str,
    pub user_template: &'static str,
}

pub fn get_prompt(kind: NarrativeKind) -> PromptTemplate {
    match kind {
        NarrativeKind::Overview => PromptTemplate {
            system: "You are a senior developer explaining a codebase to a newcomer. Be concise, engaging, and beginner-friendly. Use analogies where helpful.",
            user_template: "Given the following codebase context, write a 2-3 paragraph overview explaining what this project does, who it's for, and why it matters.\n\nContext:\n{context}",
        },
        NarrativeKind::Architecture => PromptTemplate {
            system: "You are a software architect explaining system design. Focus on module responsibilities and how they interact. Use simple analogies like 'think of module X as the post office that routes messages.'",
            user_template: "Given the following community/module structure and their connections, explain how this codebase is structured in 3-5 paragraphs. Mention the key modules, their responsibilities, and how they interact.\n\nContext:\n{context}",
        },
        NarrativeKind::ReadingOrder => PromptTemplate {
            system: "You are a mentor guiding a new developer through a codebase. Recommend what to read first and why.",
            user_template: "Given the following codebase analysis (entry points, fan-in scores, community centrality), recommend the 3-5 most important files to read first to understand 80% of the codebase. For each file, explain WHY it's important in 1-2 sentences.\n\nContext:\n{context}",
        },
        NarrativeKind::ExtensionGuide => PromptTemplate {
            system: "You are a developer advocate writing an extension guide. Be practical and step-by-step.",
            user_template: "Given the following high fan-in interfaces, plugin patterns, and configuration points, write a step-by-step guide for adding new features to this codebase.\n\nContext:\n{context}",
        },
        NarrativeKind::ContributionGuide => PromptTemplate {
            system: "You are writing a contribution guide for newcomers. Be welcoming and specific.",
            user_template: "Given the following entry points, code patterns, and test coverage information, write a guide for first-time contributors. Include how to find good first issues, understand the codebase, and submit changes.\n\nContext:\n{context}",
        },
        NarrativeKind::WhyTrending => PromptTemplate {
            system: "You are a tech journalist explaining why a project is gaining attention. Be enthusiastic but factual.",
            user_template: "Given the following project description and ecosystem context, write 1-2 paragraphs about why developers are excited about this project.\n\nContext:\n{context}",
        },
        NarrativeKind::ModuleSummary => PromptTemplate {
            system: "You are documenting a module/community for a team wiki. Use beginner-friendly analogies and keep it under 2 paragraphs.",
            user_template: "Given the following symbols, edges, and metrics for this module, write a 1-2 paragraph summary of what this module does, its key types, and how it connects to the rest of the codebase. Use beginner-friendly analogies.\n\nContext:\n{context}",
        },
        NarrativeKind::SymbolExplanation => PromptTemplate {
            system: "You are explaining a function/class to a junior developer. Be clear and use analogies if appropriate.",
            user_template: "Given the following symbol signature, its callers, callees, and containing file context, explain what this symbol does in 2-3 sentences. Include a beginner-friendly analogy if appropriate.\n\nContext:\n{context}",
        },
    }
}
