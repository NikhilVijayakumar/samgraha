//! Generic scoring primitives, one per `calculation_rules.calculation_method`
//! value (see `docs/crates-refactor-proposal.md` Phase 2). Each function is a
//! fixed, standard-agnostic numeric operation — the standard-specific data
//! (which rules, what weights, what bucket names, what bands) lives in
//! `AuditRuleDef`/`CalculationRule`/`ScoreBand`, nothing here knows about any
//! particular standard.
//!
//! Only `weighted_pass_rate`, `weighted_sum`, and `threshold_lookup` are
//! wired to real callers today (`framework.rs`) — the only three a standard's
//! deterministic-rule audit can actually exercise right now. `sum_capped_at_100`,
//! `weighted_merge`, `reliability_aware_ensemble`, and `trend_comparison` are
//! documented in the proposal but need semantic-rule execution (not yet
//! built — `StandardDefinition.audit_rules` only loads `kind = 'deterministic'`
//! rows) or historical score tracking before there's real data to compute them
//! from. Not stubbed here — a function with no real caller and no real test
//! data is a guess, not an implementation.

use schemas::audit::{CalculationInput, CalculationRule, ScoreBand};
use schemas::standard::AuditRuleDef;
use std::collections::HashMap;
use std::collections::HashSet;

/// `weighted_pass_rate`: 100 * sum(weight where passed) / sum(weight, all rules).
/// `failed_ids` are the check_ids with at least one finding against them.
pub fn weighted_pass_rate(rules: &[&AuditRuleDef], failed_ids: &HashSet<&str>) -> f64 {
    let total_weight: f64 = rules.iter().map(|r| r.weight).sum();
    if total_weight <= 0.0 {
        return 100.0;
    }
    let passed_weight: f64 = rules
        .iter()
        .filter(|r| !failed_ids.contains(r.id.as_str()))
        .map(|r| r.weight)
        .sum();
    (passed_weight / total_weight) * 100.0
}

/// `weighted_sum`: bucket-weighted sum (e.g. final_score's 4 x 25 buckets).
/// A bucket named in `inputs` but missing from `bucket_scores` defaults to
/// 100.0 — a bucket that never ran isn't a penalty, it's absent data.
pub fn weighted_sum(inputs: &[CalculationInput], bucket_scores: &HashMap<String, f64>) -> f64 {
    let total_weight: f64 = inputs.iter().map(|i| i.weight).sum();
    if total_weight <= 0.0 {
        return 100.0;
    }
    inputs
        .iter()
        .map(|i| {
            let score = bucket_scores.get(&i.name).copied().unwrap_or(100.0);
            (score / 100.0) * i.weight
        })
        .sum()
}

/// `threshold_lookup`: score -> rating label via a standard's own score_bands.
pub fn threshold_lookup(score: f64, bands: &[ScoreBand]) -> Option<String> {
    bands
        .iter()
        .find(|b| score >= b.min_score && score <= b.max_score)
        .map(|b| b.rating.clone())
}

/// Find a standard's declared calculation rule for a bucket by name (e.g.
/// "deterministic_document", "deterministic_section" — the bucket-naming
/// convention `_bucket_name()` in `knowledge-hub-loader.py` derives from each
/// calculation file's own path, not a hardcoded list).
pub fn find_bucket<'a>(rules: &'a [CalculationRule], bucket: &str) -> Option<&'a CalculationRule> {
    rules.iter().find(|r| r.bucket == bucket)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, weight: f64) -> AuditRuleDef {
        AuditRuleDef {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            severity: "warning".into(),
            evidence_type: "file_presence".into(),
            scope: String::new(),
            weight,
            mandatory: false,
            params: HashMap::new(),
        }
    }

    #[test]
    fn weighted_pass_rate_all_pass_is_100() {
        let rules = vec![rule("a", 1.0), rule("b", 2.0)];
        let refs: Vec<&AuditRuleDef> = rules.iter().collect();
        let failed = HashSet::new();
        assert_eq!(weighted_pass_rate(&refs, &failed), 100.0);
    }

    #[test]
    fn weighted_pass_rate_weighs_by_rule_weight_not_count() {
        // "a" (weight 1) fails, "b" (weight 3) passes -> 3/4 * 100 = 75, not 50.
        let rules = vec![rule("a", 1.0), rule("b", 3.0)];
        let refs: Vec<&AuditRuleDef> = rules.iter().collect();
        let failed: HashSet<&str> = ["a"].into_iter().collect();
        assert_eq!(weighted_pass_rate(&refs, &failed), 75.0);
    }

    #[test]
    fn weighted_pass_rate_no_rules_defaults_to_100() {
        let failed = HashSet::new();
        assert_eq!(weighted_pass_rate(&[], &failed), 100.0);
    }

    #[test]
    fn weighted_sum_combines_buckets_by_weight() {
        let inputs = vec![
            CalculationInput { name: "deterministic_whole".into(), weight: 25.0 },
            CalculationInput { name: "deterministic_section".into(), weight: 25.0 },
            CalculationInput { name: "semantic_whole".into(), weight: 25.0 },
            CalculationInput { name: "semantic_section".into(), weight: 25.0 },
        ];
        let mut buckets = HashMap::new();
        buckets.insert("deterministic_whole".to_string(), 80.0);
        buckets.insert("deterministic_section".to_string(), 100.0);
        // semantic_whole / semantic_section absent -> default 100 each.
        let score = weighted_sum(&inputs, &buckets);
        assert_eq!(score, 0.25 * 80.0 + 0.25 * 100.0 + 0.25 * 100.0 + 0.25 * 100.0);
    }

    #[test]
    fn threshold_lookup_finds_matching_band() {
        let bands = vec![
            ScoreBand { rating: "Excellent".into(), min_score: 90.0, max_score: 100.0 },
            ScoreBand { rating: "Needs Improvement".into(), min_score: 0.0, max_score: 89.9 },
        ];
        assert_eq!(threshold_lookup(95.0, &bands), Some("Excellent".to_string()));
        assert_eq!(threshold_lookup(50.0, &bands), Some("Needs Improvement".to_string()));
        assert_eq!(threshold_lookup(-1.0, &bands), None);
    }

    #[test]
    fn find_bucket_matches_by_name_not_position() {
        let rules = vec![
            CalculationRule { bucket: "deterministic_document".into(), calculation_method: "weighted_pass_rate".into(), formula: String::new() },
            CalculationRule { bucket: "summary_final_score".into(), calculation_method: "weighted_sum".into(), formula: String::new() },
        ];
        assert_eq!(find_bucket(&rules, "summary_final_score").unwrap().calculation_method, "weighted_sum");
        assert!(find_bucket(&rules, "nonexistent").is_none());
    }
}
