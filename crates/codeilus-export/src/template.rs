/// The embedded HTML template for repo export pages.
pub const REPO_TEMPLATE: &str = include_str!("../../../export-template/index.html");

/// The embedded HTML template for the index/listing page.
pub const INDEX_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Codeilus — {{ date }}</title>
    <style>
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace; background: #0d1117; color: #c9d1d9; line-height: 1.6; padding: 2rem; }
        h1 { font-size: 1.8rem; margin-bottom: 0.5rem; color: #58a6ff; }
        .subtitle { color: #8b949e; margin-bottom: 2rem; }
        .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.5rem; }
        .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1.5rem; transition: border-color 0.2s; }
        .card:hover { border-color: #58a6ff; }
        .card h2 { font-size: 1.1rem; color: #f0f6fc; margin-bottom: 0.5rem; }
        .card h2 a { color: inherit; text-decoration: none; }
        .card h2 a:hover { text-decoration: underline; }
        .card p { color: #8b949e; font-size: 0.85rem; margin-bottom: 0.75rem; }
        .meta { display: flex; gap: 1rem; font-size: 0.75rem; color: #8b949e; }
        .badge { display: inline-block; padding: 0.15rem 0.5rem; border-radius: 12px; font-size: 0.7rem; }
    </style>
</head>
<body>
    <h1>Codeilus Daily Digest</h1>
    <p class="subtitle">{{ date }} &mdash; {{ repos | length }} repositories analyzed</p>
    <div class="grid">
    {% for repo in repos %}
        <div class="card">
            <h2><a href="{{ repo.file_path }}">{{ repo.name }}</a></h2>
            {% if repo.description %}<p>{{ repo.description }}</p>{% endif %}
            <div class="meta">
                {% if repo.language %}<span class="badge" style="background: #21262d;">{{ repo.language }}</span>{% endif %}
                <span>{{ repo.file_size_kb }} KB</span>
                <span>{{ repo.exported_at }}</span>
            </div>
        </div>
    {% endfor %}
    </div>
</body>
</html>"#;
