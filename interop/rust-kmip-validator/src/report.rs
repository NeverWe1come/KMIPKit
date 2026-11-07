use crate::models::{CaseStatus, ValidationResults};

pub struct MatrixReporter;

impl MatrixReporter {
    pub fn render_ansi_summary(results: &ValidationResults) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\n=== KMIP 2.1 Validation Matrix: Server [{}] (Lang: {}) ===\n",
            results.server_id, results.language
        ));
        out.push_str(&format!(
            "Total: {} | ✅ Passed: {} | ❌ Failed: {} | ➖ Unsupported: {}\n",
            results.summary.total_cases,
            results.summary.passed,
            results.summary.failed,
            results.summary.unsupported_by_server
        ));
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        out.push_str(&format!(
            "{:<35} | {:<25} | {:<10}\n",
            "Case ID", "Operation", "Status"
        ));
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );

        for r in &results.results {
            let symbol = match r.status {
                CaseStatus::Passed => "✅ PASS",
                CaseStatus::Failed => "❌ FAIL",
                CaseStatus::UnsupportedSkipped => "➖ UNSUPP",
                CaseStatus::UnsupportedVerifiedRejected => "➖ REJECT",
                CaseStatus::UnexpectedSuccess => "⚠️ UNEXP",
            };
            out.push_str(&format!(
                "{:<35} | {:<25} | {:<10}\n",
                r.case_id, r.operation, symbol
            ));
        }
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        out
    }

    pub fn render_markdown(results: &ValidationResults) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# KMIP 2.1 Conformance Matrix: `{}`\n\n",
            results.server_id
        ));
        md.push_str(&format!("- **Language**: `{}`\n", results.language));
        md.push_str(&format!("- **Run ID**: `{}`\n", results.run_id));
        md.push_str(&format!("- **Timestamp**: `{}`\n\n", results.timestamp));
        md.push_str("### Summary\n\n");
        md.push_str("| Metric | Count |\n|---|---:|\n");
        md.push_str(&format!(
            "| Total Cases | {} |\n",
            results.summary.total_cases
        ));
        md.push_str(&format!("| ✅ Passed | {} |\n", results.summary.passed));
        md.push_str(&format!("| ❌ Failed | {} |\n", results.summary.failed));
        md.push_str(&format!(
            "| ➖ Unsupported | {} |\n\n",
            results.summary.unsupported_by_server
        ));

        md.push_str("### Detailed Operations\n\n");
        md.push_str(
            "| Case ID | Operation | Status | Encodings | API Levels |\n|---|---|:---:|---|---|\n",
        );
        for r in &results.results {
            let sym = match r.status {
                CaseStatus::Passed => "✅",
                CaseStatus::Failed => "❌",
                CaseStatus::UnsupportedSkipped | CaseStatus::UnsupportedVerifiedRejected => "➖",
                CaseStatus::UnexpectedSuccess => "⚠️",
            };
            md.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                r.case_id,
                r.operation,
                sym,
                r.encodings_verified.join(", "),
                r.api_levels_verified.join(", ")
            ));
        }
        md
    }

    pub fn render_html(results: &ValidationResults) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n<title>KMIP 2.1 Validation Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; margin: 2rem; background: #f8fafc; color: #1e293b; }\n");
        html.push_str(".card { background: white; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }\n");
        html.push_str("table { width: 100%; border-collapse: collapse; margin-top: 1rem; }\n");
        html.push_str("th, td { text-align: left; padding: 0.75rem 1rem; border-bottom: 1px solid #e2e8f0; }\n");
        html.push_str("th { background: #f1f5f9; font-weight: 600; }\n");
        html.push_str(".pass { color: #16a34a; font-weight: bold; }\n");
        html.push_str(".fail { color: #dc2626; font-weight: bold; }\n");
        html.push_str(".unsupp { color: #64748b; font-weight: bold; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str("<div class=\"card\">\n");
        html.push_str(&format!(
            "<h1>KMIP 2.1 Validation Matrix: {}</h1>\n",
            results.server_id
        ));
        html.push_str(&format!(
            "<p><strong>Language:</strong> {} | <strong>Run ID:</strong> {}</p>\n",
            results.language, results.run_id
        ));
        html.push_str(&format!("<p>✅ <strong>Passed:</strong> {} | ❌ <strong>Failed:</strong> {} | ➖ <strong>Unsupported:</strong> {}</p>\n",
            results.summary.passed, results.summary.failed, results.summary.unsupported_by_server));
        html.push_str("</div>\n<div class=\"card\">\n<table>\n<thead>\n<tr><th>Case ID</th><th>Operation</th><th>Status</th><th>Encodings</th><th>API Levels</th></tr>\n</thead>\n<tbody>\n");

        for r in &results.results {
            let (cls, sym) = match r.status {
                CaseStatus::Passed => ("pass", "✅ Passed"),
                CaseStatus::Failed => ("fail", "❌ Failed"),
                CaseStatus::UnsupportedSkipped | CaseStatus::UnsupportedVerifiedRejected => {
                    ("unsupp", "➖ Unsupported")
                }
                CaseStatus::UnexpectedSuccess => ("fail", "⚠️ Unexpected Success"),
            };
            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td>{}</td></tr>\n",
                r.case_id, r.operation, cls, sym, r.encodings_verified.join(", "), r.api_levels_verified.join(", ")
            ));
        }

        html.push_str("</tbody>\n</table>\n</div>\n</body>\n</html>\n");
        html
    }
}
