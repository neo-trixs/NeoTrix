type StudySectionTuple<'a> = (&'a str, &'a str, Vec<(String, f64)>);

#[derive(Debug, Clone)]
pub struct CaseStudySection {
    pub title: String,
    pub content: String,
    pub metrics: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub struct CaseStudy {
    pub title: String,
    pub tagline: String,
    pub problem: CaseStudySection,
    pub process: Vec<CaseStudySection>,
    pub results: Vec<CaseStudySection>,
    pub key_takeaways: Vec<String>,
}

impl CaseStudy {
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.title);
        md.push_str(&format!("> {}\n\n", self.tagline));

        md.push_str("## Problem\n\n");
        md.push_str(&format!("{}\n\n", self.problem.content));
        if !self.problem.metrics.is_empty() {
            md.push_str("| Metric | Value |\n|--------|-------|\n");
            for (k, v) in &self.problem.metrics {
                md.push_str(&format!("| {} | {:.2} |\n", k, v));
            }
            md.push('\n');
        }

        md.push_str("## Process\n\n");
        for (i, step) in self.process.iter().enumerate() {
            md.push_str(&format!("### Step {}: {}\n\n{}\n\n", i + 1, step.title, step.content));
            if !step.metrics.is_empty() {
                for (k, v) in &step.metrics {
                    md.push_str(&format!("- **{}**: {:.2}\n", k, v));
                }
                md.push('\n');
            }
        }

        md.push_str("## Results\n\n");
        for result in &self.results {
            md.push_str(&format!("### {}\n\n{}\n\n", result.title, result.content));
            if !result.metrics.is_empty() {
                md.push_str("| Metric | Value |\n|--------|-------|\n");
                for (k, v) in &result.metrics {
                    md.push_str(&format!("| {} | {:.2} |\n", k, v));
                }
                md.push('\n');
            }
        }

        md.push_str("## Key Takeaways\n\n");
        for t in &self.key_takeaways {
            md.push_str(&format!("- {}\n", t));
        }
        md.push('\n');

        md
    }
}

pub struct CaseStudyWriter;

impl Default for CaseStudyWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl CaseStudyWriter {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write<'a>(
        &self,
        title: &'a str,
        tagline: &'a str,
        problem_desc: &'a str,
        problem_metrics: Vec<(String, f64)>,
        process_steps: Vec<StudySectionTuple<'a>>,
        result_sections: Vec<StudySectionTuple<'a>>,
        takeaways: Vec<String>,
    ) -> CaseStudy {
        let process: Vec<CaseStudySection> = process_steps
            .into_iter()
            .map(|(title, content, metrics)| CaseStudySection {
                title: title.to_string(),
                content: content.to_string(),
                metrics,
            })
            .collect();

        let results: Vec<CaseStudySection> = result_sections
            .into_iter()
            .map(|(title, content, metrics)| CaseStudySection {
                title: title.to_string(),
                content: content.to_string(),
                metrics,
            })
            .collect();

        CaseStudy {
            title: title.to_string(),
            tagline: tagline.to_string(),
            problem: CaseStudySection {
                title: "Problem".to_string(),
                content: problem_desc.to_string(),
                metrics: problem_metrics,
            },
            process,
            results,
            key_takeaways: takeaways,
        }
    }

    pub fn simple(&self, title: &str, problem: &str, solution: &str, outcome: &str) -> CaseStudy {
        CaseStudy {
            title: title.to_string(),
            tagline: format!("{} → {} → {}", problem, solution, outcome),
            problem: CaseStudySection {
                title: "Problem".to_string(),
                content: problem.to_string(),
                metrics: vec![],
            },
            process: vec![CaseStudySection {
                title: "Solution".to_string(),
                content: solution.to_string(),
                metrics: vec![],
            }],
            results: vec![CaseStudySection {
                title: "Outcome".to_string(),
                content: outcome.to_string(),
                metrics: vec![],
            }],
            key_takeaways: vec!["Documented for future reference.".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_case_study() {
        let writer = CaseStudyWriter::new();
        let cs = writer.simple("Migration", "Legacy system slow", "Moved to cloud", "40% faster");
        assert_eq!(cs.title, "Migration");
        assert_eq!(cs.process.len(), 1);
        assert_eq!(cs.results.len(), 1);
    }

    #[test]
    fn test_full_case_study() {
        let writer = CaseStudyWriter::new();
        let cs = writer.write(
            "AI Pipeline Optimization",
            "Reducing inference latency by 60%",
            "Model inference took 2.3s on average",
            vec![("Latency".to_string(), 2300.0)],
            vec![
                ("Quantization", "Converted FP32 to INT8", vec![("Model size reduction".to_string(), 0.75)]),
                ("Batching", "Implemented dynamic batching", vec![("Throughput increase".to_string(), 3.5)]),
            ],
            vec![
                ("Latency", "Reduced to 920ms", vec![("Improvement".to_string(), 0.6)]),
                ("Cost", "40% reduction in compute costs", vec![("Savings".to_string(), 0.4)]),
            ],
            vec!["Quantization has no measurable accuracy loss.".to_string()],
        );
        assert_eq!(cs.title, "AI Pipeline Optimization");
        assert_eq!(cs.process.len(), 2);
        assert_eq!(cs.results.len(), 2);
        assert_eq!(cs.key_takeaways.len(), 1);
    }

    #[test]
    fn test_markdown_output() {
        let writer = CaseStudyWriter::new();
        let cs = writer.simple("Test", "Problem X", "Solution Y", "Outcome Z");
        let md = cs.to_markdown();
        assert!(md.contains("# Test"));
        assert!(md.contains("Problem X"));
        assert!(md.contains("Solution Y"));
        assert!(md.contains("Outcome Z"));
        assert!(md.contains("## Key Takeaways"));
    }

    #[test]
    fn test_full_markdown() {
        let writer = CaseStudyWriter::new();
        let cs = writer.write(
            "Full Study",
            "Tagline here",
            "Problem description",
            vec![("Metric A".to_string(), 10.0)],
            vec![("Step 1", "Did X", vec![("Impact".to_string(), 0.5)])],
            vec![("Result A", "Got Y", vec![])],
            vec!["Learning 1".to_string()],
        );
        let md = cs.to_markdown();
        assert!(md.contains("## Problem"));
        assert!(md.contains("## Process"));
        assert!(md.contains("## Results"));
        assert!(md.contains("### Step 1: Step 1"));
        assert!(md.contains("Metric A"));
    }

    #[test]
    fn test_writer_new() {
        let writer = CaseStudyWriter::new();
        let cs = writer.simple("A", "B", "C", "D");
        assert_eq!(cs.problem.content, "B");
    }

    #[test]
    fn test_takeaways_in_markdown() {
        let writer = CaseStudyWriter::new();
        let cs = writer.simple("X", "P", "S", "O");
        let md = cs.to_markdown();
        assert!(md.contains("Documented for future reference."));
    }
}
