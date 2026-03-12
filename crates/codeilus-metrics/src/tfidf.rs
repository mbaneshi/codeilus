use std::collections::HashMap;

/// Split a symbol name into tokens (handles camelCase, PascalCase, ALLCAPS, and snake_case).
pub fn tokenize(name: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = name.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];

        if ch == '_' || ch == '-' || ch == '.' {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        } else if ch.is_uppercase() {
            // Transition to uppercase: check if this is a new word boundary
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            let in_upper_run = i > 0 && chars[i - 1].is_uppercase();

            if prev_lower || (in_upper_run && next_lower) {
                // New word boundary: "parseJSON" at F, or "JSONFile" at F
                if !current.is_empty() {
                    tokens.push(current.to_lowercase());
                    current.clear();
                }
            }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    // Filter out very short tokens
    tokens.into_iter().filter(|t| t.len() >= 2).collect()
}

/// Compute TF-IDF keywords for each community.
///
/// Returns top `limit` keywords per community sorted by TF-IDF score.
pub fn compute_tfidf(
    community_names: &[Vec<String>],
    limit: usize,
) -> Vec<Vec<(String, f64)>> {
    let n_communities = community_names.len();
    if n_communities == 0 {
        return Vec::new();
    }

    // Tokenize all names in each community
    let community_tokens: Vec<Vec<String>> = community_names
        .iter()
        .map(|names| {
            names
                .iter()
                .flat_map(|name| tokenize(name))
                .collect()
        })
        .collect();

    // Document frequency: how many communities contain each token
    let mut df: HashMap<String, usize> = HashMap::new();
    for tokens in &community_tokens {
        let unique: std::collections::HashSet<&String> = tokens.iter().collect();
        for token in unique {
            *df.entry(token.clone()).or_default() += 1;
        }
    }

    // Compute TF-IDF per community
    community_tokens
        .iter()
        .map(|tokens| {
            // Term frequency
            let mut tf: HashMap<String, usize> = HashMap::new();
            for token in tokens {
                *tf.entry(token.clone()).or_default() += 1;
            }

            let total = tokens.len() as f64;
            if total == 0.0 {
                return Vec::new();
            }

            let mut scores: Vec<(String, f64)> = tf
                .iter()
                .map(|(term, &count)| {
                    let tf_val = count as f64 / total;
                    let df_val = df.get(term).copied().unwrap_or(1) as f64;
                    let idf_val = (n_communities as f64 / df_val).ln() + 1.0;
                    (term.clone(), tf_val * idf_val)
                })
                .collect();

            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scores.truncate(limit);
            scores
        })
        .collect()
}
