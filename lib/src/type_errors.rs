use oxc_semantic::Semantic;
use oxc_diagnostics::OxcDiagnostic;
use oxc_ast::ast::Program;
use oxc_ast::visit::Visit;
use crate::models::TypeError;
use crate::visitors::type_error_visitor::TypeErrorVisitor;

pub fn extract_type_errors<'a>(
    source: &'a str,
    semantic: &'a Semantic<'a>,
    diagnostics: &'a Vec<OxcDiagnostic>,
    program: &Program<'a>,
    file_path: String,
) -> Vec<TypeError> {
    let mut visitor = TypeErrorVisitor::new(source, semantic, diagnostics);
    visitor.visit_program(program);
    
    let mut errors = visitor.errors;
    for error in &mut errors {
        error.file = file_path.clone();
    }
    errors
}
