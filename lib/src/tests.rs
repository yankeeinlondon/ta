use oxc_ast::ast::Program;
use oxc_ast::visit::Visit;
use crate::models::TypeTest;
use crate::visitors::test_visitor::TestVisitor;

pub fn extract_tests<'a>(
    program: &Program<'a>,
    file_path: String,
) -> Vec<TypeTest> {
    let mut visitor = TestVisitor::new(file_path);
    visitor.visit_program(program);
    visitor.tests
}
