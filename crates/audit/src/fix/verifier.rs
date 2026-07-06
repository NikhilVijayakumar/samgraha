use crate::fix::types::{Verdict, VerificationDetail};
use anyhow::Result;
use std::collections::HashMap;

type CheckRunner = Box<dyn Fn(&str, &str) -> Result<f64>>;

pub struct Verifier {
    run_check: CheckRunner,
    /// Dependency graph keyed by `(domain, check_id)` → dependent check_ids.
    dependency_graph: HashMap<(String, String), Vec<String>>,
}

impl Verifier {
    pub fn new(run_check: CheckRunner) -> Self {
        Self {
            run_check,
            dependency_graph: HashMap::new(),
        }
    }

    pub fn add_dependency(
        &mut self,
        domain: &str,
        check_id: &str,
        depends_on: Vec<String>,
    ) {
        self.dependency_graph
            .insert((domain.to_string(), check_id.to_string()), depends_on);
    }

    pub fn verify(
        &self,
        domain: &str,
        check_ids: &[String],
    ) -> Result<Verdict> {
        let mut all_check_ids = check_ids.to_vec();

        for cid in check_ids {
            if let Some(deps) = self.dependency_graph.get(&(domain.to_string(), cid.clone())) {
                for dep in deps {
                    if !all_check_ids.contains(dep) {
                        all_check_ids.push(dep.clone());
                    }
                }
            }
        }

        let mut details = Vec::new();
        let mut total_score = 0.0f64;
        let mut passed = true;

        for cid in &all_check_ids {
            match (self.run_check)(domain, cid) {
                Ok(score) => {
                    total_score += score;
                    details.push(VerificationDetail {
                        check_id: cid.clone(),
                        domain: domain.to_string(),
                        score,
                        message: if score >= 9.0 {
                            "Passed".into()
                        } else {
                            format!("Score {} below threshold 9.0", score)
                        },
                    });
                    if score < 9.0 {
                        passed = false;
                    }
                }
                Err(e) => {
                    details.push(VerificationDetail {
                        check_id: cid.clone(),
                        domain: domain.to_string(),
                        score: 0.0,
                        message: format!("Check failed: {}", e),
                    });
                    passed = false;
                }
            }
        }

        let avg = if all_check_ids.is_empty() {
            0.0
        } else {
            total_score / all_check_ids.len() as f64
        };

        let check_scores = details
            .iter()
            .map(|d| {
                let key = format!("{}.{}", d.domain, d.check_id);
                (key, d.score)
            })
            .collect();

        Ok(Verdict {
            score: avg,
            check_scores,
            details,
            passed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_runner(results: Vec<(&'static str, f64)>) -> CheckRunner {
        let map: HashMap<String, f64> = results.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        Box::new(move |domain, check_id| {
            let key = format!("{}.{}", domain, check_id);
            map.get(&key).copied().ok_or_else(|| anyhow::anyhow!("No mock result for {}", key))
        })
    }

    #[test]
    fn single_check_passes() {
        let v = Verifier::new(mock_runner(vec![("build.B1", 9.5)]));
        let verdict = v.verify("build", &["B1".into()]).unwrap();
        assert!(verdict.passed);
        assert_eq!(verdict.score, 9.5);
        assert_eq!(verdict.details.len(), 1);
    }

    #[test]
    fn single_check_fails_below_threshold() {
        let v = Verifier::new(mock_runner(vec![("build.B1", 5.0)]));
        let verdict = v.verify("build", &["B1".into()]).unwrap();
        assert!(!verdict.passed);
        assert_eq!(verdict.score, 5.0);
    }

    #[test]
    fn multiple_checks_averaged() {
        let v = Verifier::new(mock_runner(vec![
            ("build.B1", 10.0),
            ("build.B2", 8.0),
        ]));
        let verdict = v.verify("build", &["B1".into(), "B2".into()]).unwrap();
        assert!(!verdict.passed);
        assert!((verdict.score - 9.0).abs() < 0.01);
    }

    #[test]
    fn dependency_graph_included() {
        let mut v = Verifier::new(mock_runner(vec![
            ("build.B1", 10.0),
            ("build.B2", 9.0),
            ("build.B3", 8.0),
        ]));
        v.add_dependency("build", "B1", vec!["B3".into()]);
        let verdict = v.verify("build", &["B1".into()]).unwrap();
        // Should have run B1 and B3 (dependency), so 2 checks averaged
        assert!((verdict.score - 9.0).abs() < 0.01);
        assert_eq!(verdict.details.len(), 2);
    }

    #[test]
    fn check_runner_error_returns_zero() {
        let v = Verifier::new(Box::new(|_, _| {
            Err(anyhow::anyhow!("check crashed"))
        }));
        let verdict = v.verify("build", &["B1".into()]).unwrap();
        assert!(!verdict.passed);
        assert_eq!(verdict.score, 0.0);
        assert!(verdict.details[0].message.contains("check crashed"));
    }

    #[test]
    fn empty_checks_returns_zero() {
        let v = Verifier::new(Box::new(|_, _| Ok(10.0)));
        let verdict = v.verify("build", &[]).unwrap();
        assert!(verdict.passed);
        assert_eq!(verdict.score, 0.0);
    }
}
