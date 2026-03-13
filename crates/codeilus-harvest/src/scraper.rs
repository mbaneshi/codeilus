//! GitHub trending page scraper.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use scraper::{Html, Selector};
use tracing::{debug, warn};

use crate::types::{HarvestConfig, HarvestStatus, HarvestedRepo};

/// Build the URL for the GitHub trending page.
pub fn build_trending_url(config: &HarvestConfig) -> String {
    let mut url = "https://github.com/trending".to_string();
    if let Some(lang) = &config.language {
        url.push('/');
        url.push_str(lang);
    }
    url.push_str(&format!("?since={}", config.since.as_str()));
    url
}

/// Scrape the GitHub trending page and return discovered repos.
pub async fn scrape_trending(config: &HarvestConfig) -> CodeilusResult<Vec<HarvestedRepo>> {
    let url = build_trending_url(config);
    debug!(url = %url, "fetching trending page");

    let response = reqwest::get(&url).await.map_err(|e| {
        CodeilusError::Harvest(format!("Failed to fetch trending page: {}", e))
    })?;

    if !response.status().is_success() {
        return Err(CodeilusError::Harvest(format!(
            "GitHub trending returned status {}",
            response.status()
        )));
    }

    let html = response.text().await.map_err(|e| {
        CodeilusError::Harvest(format!("Failed to read response body: {}", e))
    })?;

    let repos = parse_trending_html(&html);
    let limited = repos.into_iter().take(config.max_repos).collect();
    Ok(limited)
}

/// Parse trending HTML into HarvestedRepo structs.
///
/// This is separated from scrape_trending for testability with fixture HTML.
pub fn parse_trending_html(html: &str) -> Vec<HarvestedRepo> {
    let document = Html::parse_document(html);
    let mut repos = Vec::new();

    // GitHub trending uses article.Box-row for each repo entry
    let row_selector = Selector::parse("article.Box-row").unwrap_or_else(|_| {
        warn!("Could not parse article.Box-row selector, trying fallback");
        Selector::parse("article").unwrap()
    });

    let h2_a_selector = Selector::parse("h2 a").unwrap();
    let desc_selector = Selector::parse("p").unwrap();
    let lang_selector = Selector::parse("span[itemprop='programmingLanguage']").unwrap();

    for row in document.select(&row_selector) {
        // Extract owner/name from h2 a href
        let (owner, name) = match row.select(&h2_a_selector).next() {
            Some(a) => {
                let href = a.value().attr("href").unwrap_or("");
                let parts: Vec<&str> = href.trim_matches('/').split('/').collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    warn!(href = %href, "unexpected href format in trending row");
                    continue;
                }
            }
            None => {
                warn!("no h2 a found in trending row");
                continue;
            }
        };

        // Description
        let description = row
            .select(&desc_selector)
            .next()
            .map(|p| p.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        // Language
        let language = row
            .select(&lang_selector)
            .next()
            .map(|span| span.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        // Stars today — look for text containing "stars today" or "stars this"
        let stars_today = extract_stars_today(&row);

        // Total stars — first link with star-related content
        let total_stars = extract_total_stars(&row);

        let full_name = format!("{}/{}", owner, name);
        let url = format!("https://github.com/{}", full_name);
        let clone_url = format!("https://github.com/{}.git", full_name);

        repos.push(HarvestedRepo {
            owner,
            name,
            full_name,
            description,
            language,
            stars_today,
            total_stars,
            url,
            clone_url,
            fingerprint: String::new(), // computed after clone
            status: HarvestStatus::Found,
        });
    }

    debug!(count = repos.len(), "parsed trending repos");
    repos
}

fn extract_stars_today(row: &scraper::ElementRef) -> Option<usize> {
    // Look for text like "123 stars today" or "456 stars this week"
    let text = row.text().collect::<String>();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("stars today")
            || trimmed.contains("stars this week")
            || trimmed.contains("stars this month")
        {
            // Extract the number
            let num_str: String = trimmed.chars().filter(|c| c.is_ascii_digit() || *c == ',').collect();
            let num_str = num_str.replace(',', "");
            if let Ok(n) = num_str.parse() {
                return Some(n);
            }
        }
    }
    None
}

fn extract_total_stars(row: &scraper::ElementRef) -> Option<usize> {
    // Look for links with star counts (usually the first .Link--muted with a number)
    let a_selector = Selector::parse("a.Link--muted").unwrap();
    for a in row.select(&a_selector) {
        let text: String = a.text().collect::<String>().trim().to_string();
        let num_str: String = text.chars().filter(|c| c.is_ascii_digit() || *c == ',').collect();
        let num_str = num_str.replace(',', "");
        if let Ok(n) = num_str.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TrendingSince;

    #[test]
    fn scrape_url_construction() {
        let config = HarvestConfig {
            language: Some("rust".to_string()),
            since: TrendingSince::Weekly,
            ..Default::default()
        };
        let url = build_trending_url(&config);
        assert_eq!(url, "https://github.com/trending/rust?since=weekly");

        let config_default = HarvestConfig::default();
        let url2 = build_trending_url(&config_default);
        assert_eq!(url2, "https://github.com/trending?since=daily");
    }

    #[test]
    fn scrape_parse_html() {
        let html = include_str!("../tests/fixtures/trending.html");
        let repos = parse_trending_html(html);
        assert!(
            repos.len() >= 3,
            "Expected at least 3 repos from fixture, got {}",
            repos.len()
        );

        // Check first repo has owner and name
        let first = &repos[0];
        assert!(!first.owner.is_empty(), "Owner should not be empty");
        assert!(!first.name.is_empty(), "Name should not be empty");
        assert!(
            first.url.starts_with("https://github.com/"),
            "URL should start with https://github.com/"
        );
        assert!(
            first.clone_url.ends_with(".git"),
            "Clone URL should end with .git"
        );
        assert_eq!(first.status, HarvestStatus::Found);
    }

    #[test]
    fn scrape_missing_fields() {
        // HTML with minimal repo entry — missing description and language
        let html = r#"
        <html><body>
        <article class="Box-row">
            <h2><a href="/testowner/testrepo">testowner / testrepo</a></h2>
        </article>
        </body></html>
        "#;
        let repos = parse_trending_html(html);
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].owner, "testowner");
        assert_eq!(repos[0].name, "testrepo");
        assert!(repos[0].description.is_none());
        assert!(repos[0].language.is_none());
        assert!(repos[0].stars_today.is_none());
    }
}
