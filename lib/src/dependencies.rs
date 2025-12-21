use std::path::PathBuf;
use oxc_ast::ast::Program;
use oxc_ast::visit::Visit;
use crate::visitors::dependency_visitor::DependencyVisitor;

pub fn extract_dependencies<'a>(
    program: &Program<'a>,
    file_path: PathBuf,
) -> Vec<String> {
    let mut visitor = DependencyVisitor::new(file_path);
    visitor.visit_program(program);
    visitor.dependencies
}
