use serde::{Deserialize, Serialize};

/// One test suite's results (unit or e2e) — the shape a repo's own
/// test-running script must emit as JSON so audit can read real pass/fail
/// counts instead of guessing from file presence.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TestSuiteResult {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    #[serde(default)]
    pub skipped: u32,
    #[serde(default)]
    pub failures: Vec<TestFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TestFailure {
    pub name: String,
    #[serde(default)]
    pub message: String,
}

/// Structured results a repo-declared `[pipelines.test]` script must write
/// to its first declared `artifacts` path — MCP's requirement, script is
/// repo-supplied (cargo test + tarpaulin, pytest + coverage.py, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TestRunReport {
    #[serde(default)]
    pub unit: TestSuiteResult,
    #[serde(default)]
    pub e2e: TestSuiteResult,
    #[serde(default)]
    pub coverage_percent: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_documented_shape() {
        let json = r#"{
            "unit": { "total": 42, "passed": 40, "failed": 2, "skipped": 0, "failures": [{"name": "test_foo", "message": "assertion failed"}] },
            "e2e":  { "total": 5,  "passed": 5,  "failed": 0, "skipped": 0, "failures": [] },
            "coverage_percent": 78.4
        }"#;
        let report: TestRunReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.unit.total, 42);
        assert_eq!(report.unit.failed, 2);
        assert_eq!(report.unit.failures[0].name, "test_foo");
        assert_eq!(report.e2e.passed, 5);
        assert_eq!(report.coverage_percent, Some(78.4));

        let round_tripped: TestRunReport =
            serde_json::from_str(&serde_json::to_string(&report).unwrap()).unwrap();
        assert_eq!(round_tripped, report);
    }

    #[test]
    fn unknown_field_is_rejected_not_silently_ignored() {
        // A typo'd field name (e.g. "pased" instead of "passed") must fail
        // loudly rather than silently defaulting `passed` to 0 and
        // misreporting every test as failed.
        let json = r#"{"unit": {"total": 1, "pased": 1, "failed": 0}}"#;
        assert!(serde_json::from_str::<TestRunReport>(json).is_err());
    }

    #[test]
    fn missing_fields_default_sanely() {
        let report: TestRunReport = serde_json::from_str("{}").unwrap();
        assert_eq!(report.unit.total, 0);
        assert_eq!(report.coverage_percent, None);
    }
}
