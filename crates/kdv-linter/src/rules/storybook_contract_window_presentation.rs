use crate::diagnostics::Violation;
use crate::span::SpanOps;
use crate::workspace::SourceFile;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(super) struct WindowPresentationChecker<'a> {
    file: &'a SourceFile,
}

impl<'a> WindowPresentationChecker<'a> {
    pub(super) fn new(file: &'a SourceFile) -> Self {
        Self { file }
    }

    pub(super) fn violations(&self) -> Vec<Violation> {
        if self.is_test_source() {
            return Vec::new();
        }
        let mut visitor = WindowPresentationVisitor::new(self.file.path().to_path_buf());
        visitor.visit_file(self.file.syntax());
        let mut violations = visitor.into_violations();
        violations.extend(self.source_pattern_violations());
        violations
    }

    fn is_test_source(&self) -> bool {
        self.file.path_contains("/tests/")
            || self.file.path_ends_with("_tests.rs")
            || self.file.path_ends_with("test_support.rs")
    }

    fn source_pattern_violations(&self) -> Vec<Violation> {
        self.file
            .source()
            .lines()
            .enumerate()
            .flat_map(|(index, line)| self.violations_for_line(index, line))
            .collect()
    }

    fn violations_for_line(&self, index: usize, line: &str) -> Vec<Violation> {
        WindowPresentationPattern::all()
            .iter()
            .filter_map(|pattern| pattern.column(line).map(|column| (*pattern, column)))
            .map(|(pattern, column)| self.violation(index + 1, column + 1, pattern.message()))
            .collect()
    }

    fn violation(&self, line: usize, column: usize, message: &'static str) -> Violation {
        Violation::new(PathBuf::from(self.file.path()), line, column, RULE, message)
    }
}

#[derive(Clone, Copy)]
enum WindowPresentationPattern {
    ImportRawPresentationList,
    ImportRawPresentationTail,
    ManualScaleThreshold,
}

impl WindowPresentationPattern {
    fn all() -> &'static [Self] {
        &[
            Self::ImportRawPresentationList,
            Self::ImportRawPresentationTail,
            Self::ManualScaleThreshold,
        ]
    }

    fn column(self, line: &str) -> Option<usize> {
        match self {
            Self::ImportRawPresentationList => line.find("present_frame,"),
            Self::ImportRawPresentationTail => line.find("present_frame}"),
            Self::ManualScaleThreshold => line.find("scale_factor() > 1.0"),
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::ImportRawPresentationList | Self::ImportRawPresentationTail => {
                "Storybook must not import raw frame presentation; use KUC present_frame_for_window."
            }
            Self::ManualScaleThreshold => {
                "Storybook must not recreate physical/logical scale presentation rules."
            }
        }
    }
}

struct WindowPresentationVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl WindowPresentationVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn push(&mut self, span: proc_macro2::Span, message: &'static str) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            RULE,
            message,
        ));
    }

    fn call_name(node: &syn::ExprCall) -> Option<&syn::Ident> {
        let syn::Expr::Path(path) = node.func.as_ref() else {
            return None;
        };
        path.path.segments.last().map(|segment| &segment.ident)
    }
}

impl<'ast> Visit<'ast> for WindowPresentationVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if Self::call_name(node).is_some_and(|ident| ident == "present_frame") {
            self.push(
                node.func.span(),
                "Storybook must call KUC present_frame_for_window instead of raw frame presentation.",
            );
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.ident == "should_present_physical_frame_directly" {
            self.push(
                node.sig.ident.span(),
                "Storybook must not own physical/logical window presentation decisions.",
            );
        }
        syn::visit::visit_item_fn(self, node);
    }
}

const RULE: &str = "no_manual_window_presentation";
