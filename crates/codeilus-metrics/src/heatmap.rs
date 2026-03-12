use crate::types::FileMetrics;

/// Compute heatmap scores for all files.
///
/// Score = 0.4 * normalized_complexity + 0.3 * normalized_churn
///       + 0.2 * normalized_size + 0.1 * normalized_fan_in
///
/// All components normalized to 0.0-1.0 (min-max).
pub fn compute_heatmap(
    metrics: &mut [FileMetrics],
    fan_in_per_file: &std::collections::HashMap<codeilus_core::ids::FileId, usize>,
) {
    if metrics.is_empty() {
        return;
    }

    // Gather raw values
    let complexities: Vec<f64> = metrics.iter().map(|m| m.complexity).collect();
    let churns: Vec<f64> = metrics.iter().map(|m| m.churn as f64).collect();
    let sizes: Vec<f64> = metrics.iter().map(|m| m.sloc as f64).collect();
    let fan_ins: Vec<f64> = metrics
        .iter()
        .map(|m| fan_in_per_file.get(&m.file_id).copied().unwrap_or(0) as f64)
        .collect();

    // Min-max normalize
    let norm_complexity = normalize(&complexities);
    let norm_churn = normalize(&churns);
    let norm_size = normalize(&sizes);
    let norm_fan_in = normalize(&fan_ins);

    for (i, m) in metrics.iter_mut().enumerate() {
        m.heatmap_score = 0.4 * norm_complexity[i]
            + 0.3 * norm_churn[i]
            + 0.2 * norm_size[i]
            + 0.1 * norm_fan_in[i];
    }
}

fn normalize(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    if range == 0.0 {
        return vec![0.0; values.len()];
    }

    values.iter().map(|v| (v - min) / range).collect()
}
