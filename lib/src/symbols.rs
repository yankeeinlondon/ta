use oxc_ast::ast::Program;
use oxc_ast::visit::Visit;
use crate::models::SymbolInfo;
use crate::visitors::symbol_visitor::SymbolVisitor;

pub fn extract_symbols<'a>(
    source: &'a str,
    program: &Program<'a>,
    file_path: String,
    exported_only: bool,
) -> Vec<SymbolInfo> {
    let mut visitor = SymbolVisitor::new(source, file_path, exported_only);
    visitor.visit_program(program);
    visitor.symbols
}
