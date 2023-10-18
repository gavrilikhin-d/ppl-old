use miette::Report;

/// Struct to report errors
pub struct Reporter {
    /// Reports reported through this reporter
    pub reports: Vec<Report>,
    /// Handler for reports
    pub report_handler: Box<dyn FnMut(&Report)>,
}

impl Reporter {
    /// Create new reporter
    pub fn new() -> Self {
        Self {
            reports: vec![],
            report_handler: Box::new(|r| println!("{:?}", r)),
        }
    }

    /// Has this reporter reported any errors?
    pub fn has_errors(&self) -> bool {
        self.reports.iter().any(|r| {
            r.severity().is_none()
                || r.severity()
                    .is_some_and(|s| matches!(s, miette::Severity::Error))
        })
    }

    /// Handle a new report
    pub fn report(&mut self, report: impl Into<Report>) -> &mut Self {
        let report = report.into();
        (self.report_handler)(&report);
        self.reports.push(report);
        self
    }
}
