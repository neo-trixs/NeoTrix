use super::{
    AccessibilityAntiPatternDetector, AntiPatternDetector, ColorAntiPatternDetector,
    DesignViolation, LayoutAntiPatternDetector, MotionAntiPatternDetector,
    TypographyAntiPatternDetector,
};

pub struct InnerCritic {
    detectors: Vec<Box<dyn AntiPatternDetector>>,
}

impl Default for InnerCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerCritic {
    pub fn new() -> Self {
        let detectors: Vec<Box<dyn AntiPatternDetector>> = vec![
            Box::new(ColorAntiPatternDetector),
            Box::new(TypographyAntiPatternDetector),
            Box::new(LayoutAntiPatternDetector),
            Box::new(MotionAntiPatternDetector),
            Box::new(AccessibilityAntiPatternDetector),
        ];
        Self { detectors }
    }

    pub fn audit(&self, content: &str) -> Vec<DesignViolation> {
        let mut all = Vec::new();
        for detector in &self.detectors {
            all.extend(detector.detect(content));
        }
        all.sort_by(|a, b| b.severity.cmp(&a.severity));
        all
    }

    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }
}
