use codeilus_harvest::types::*;

#[test]
fn config_defaults() {
    let config = HarvestConfig::default();
    assert_eq!(config.max_repos, 25);
    assert_eq!(config.concurrent_clones, 5);
    assert_eq!(config.since, TrendingSince::Daily);
    assert!(config.language.is_none());
}

#[test]
fn harvest_status_transitions() {
    let statuses = [
        HarvestStatus::Found,
        HarvestStatus::Cloning,
        HarvestStatus::Cloned,
        HarvestStatus::Analyzing,
        HarvestStatus::Complete,
    ];

    for status in &statuses {
        let s = status.as_str();
        let roundtrip = HarvestStatus::parse(s);
        assert_eq!(*status, roundtrip, "Roundtrip failed for {:?}", status);
    }
}

#[test]
fn skip_already_analyzed() {
    let mut repo = HarvestedRepo {
        owner: "test".to_string(),
        name: "repo".to_string(),
        full_name: "test/repo".to_string(),
        description: None,
        language: None,
        stars_today: None,
        total_stars: None,
        url: "https://github.com/test/repo".to_string(),
        clone_url: "https://github.com/test/repo.git".to_string(),
        fingerprint: "abc123".to_string(),
        status: HarvestStatus::Found,
    };

    // Simulate marking as skipped
    repo.status = HarvestStatus::Skipped;
    assert_eq!(repo.status, HarvestStatus::Skipped);
    assert_eq!(repo.status.as_str(), "skipped");
}
