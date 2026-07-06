use serde::{Deserialize, Serialize};

/// ScoreBand defines a score range with associated narrative text.
/// Shared across all audit domains for consistent report tone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBand {
    pub min: f64,
    pub max: f64,
    pub label: &'static str,
    /// Short description of what this band means for the report.
    pub description: &'static str,
    /// Overall assessment wording for executive summary.
    pub overall_wording: &'static str,
    /// Tone to use when describing findings at this level.
    pub finding_tone: &'static str,
    /// Tone for recommendations at this level.
    pub recommendation_tone: &'static str,
}

const BANDS: &[ScoreBand] = &[
    ScoreBand {
        min: 96.0, max: 100.0,
        label: "Excellent",
        description: "Comprehensive, well-structured architecture documentation",
        overall_wording: "The architecture documentation is excellent. The collection is complete, consistent, and clearly communicates system organization.",
        finding_tone: "Minor improvement opportunities",
        recommendation_tone: "Consider addressing these minor suggestions to maintain excellence",
    },
    ScoreBand {
        min: 90.0, max: 95.99,
        label: "Very Good",
        description: "Well-structured architecture with minor gaps",
        overall_wording: "The architecture documentation is very good. The collection is largely complete and consistent, with a few areas for improvement.",
        finding_tone: "Notable improvement areas",
        recommendation_tone: "Addressing these items will elevate the collection to excellent",
    },
    ScoreBand {
        min: 80.0, max: 89.99,
        label: "Good",
        description: "Solid architecture documentation with gaps",
        overall_wording: "The architecture documentation is good. Key structural elements are documented, but there are gaps in consistency, completeness, or clarity.",
        finding_tone: "Moderate findings to address",
        recommendation_tone: "Prioritize these recommendations to strengthen architectural clarity",
    },
    ScoreBand {
        min: 70.0, max: 79.99,
        label: "Acceptable",
        description: "Adequate architecture documentation needing improvement",
        overall_wording: "The architecture documentation is acceptable. The system organization is broadly described, but significant gaps exist in detail, consistency, or coverage.",
        finding_tone: "Significant findings requiring attention",
        recommendation_tone: "These items should be addressed to ensure architecture serves as a reliable foundation",
    },
    ScoreBand {
        min: 40.0, max: 69.99,
        label: "Needs Improvement",
        description: "Incomplete or inconsistent architecture documentation",
        overall_wording: "The architecture documentation needs improvement. Critical elements are missing, inconsistent, or unclear.",
        finding_tone: "Critical gaps identified",
        recommendation_tone: "These issues must be addressed for the architecture to fulfill its purpose",
    },
    ScoreBand {
        min: 0.0, max: 39.99,
        label: "Poor",
        description: "Inadequate architecture documentation",
        overall_wording: "The architecture documentation is inadequate. Essential architectural concerns are undocumented or severely inconsistent.",
        finding_tone: "Major structural deficiencies",
        recommendation_tone: "Fundamental architectural documentation work required",
    },
];

/// Returns the ScoreBand for a given score.
/// Bands are evaluated top-to-bottom; first match wins.
/// Score is clamped to [0, 100].
pub fn resolve_score_band(score: f64) -> &'static ScoreBand {
    let clamped = score.clamp(0.0, 100.0);
    for band in BANDS {
        if clamped >= band.min && clamped <= band.max {
            return band;
        }
    }
    // Fallback — should never reach here if BANDS covers 0..100
    BANDS.last().unwrap()
}

/// Returns overall assessment rating string from a score (e.g. "Excellent", "Good").
pub fn score_label(score: f64) -> &'static str {
    resolve_score_band(score).label
}

/// Build a git-style change bar string comparing current to previous score.
pub fn score_change_bar(current: f64, previous: f64) -> String {
    let diff = current - previous;
    if diff.abs() < 0.01 {
        "─".to_string()
    } else if diff > 0.0 {
        format!("▲ +{:.1}", diff)
    } else {
        format!("▼ {:.1}", diff)
    }
}

/// Returns a human-readable trend phrase.
pub fn trend_phrase(current: f64, previous: Option<f64>) -> &'static str {
    match previous {
        Some(prev) => {
            let diff = current - prev;
            if diff > 5.0 {
                "significant improvement"
            } else if diff > 0.0 {
                "slight improvement"
            } else if diff.abs() < 0.01 {
                "unchanged"
            } else if diff > -5.0 {
                "slight regression"
            } else {
                "significant regression"
            }
        }
        None => "baseline established",
    }
}
