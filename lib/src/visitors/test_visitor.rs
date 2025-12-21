use oxc_ast::visit::{walk, Visit};
use oxc_ast::ast::*;
use crate::models::{TypeTest, TestStatus};

pub struct TestVisitor {
    pub tests: Vec<TypeTest>,
    pub file_path: String,
    current_describe: Vec<String>,
}

impl TestVisitor {
    pub fn new(file_path: String) -> Self {
        Self {
            tests: Vec::new(),
            file_path,
            current_describe: Vec::new(),
        }
    }

    fn get_describe_string(&self) -> String {
        self.current_describe.join(" > ")
    }
}

impl<'a> Visit<'a> for TestVisitor {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::Identifier(ident) = &expr.callee {
            let name = ident.name.as_str();
            
            if name == "describe" {
                if let Some(Argument::StringLiteral(lit)) = expr.arguments.first() {
                    self.current_describe.push(lit.value.to_string());
                    walk::walk_call_expression(self, expr);
                    self.current_describe.pop();
                    return;
                }
            } else if name == "it" || name == "test" {
                if let Some(Argument::StringLiteral(lit)) = expr.arguments.first() {
                    let test_name = lit.value.to_string();
                    
                    // Simple heuristic for "has type cases" - look for expectTypeOf in the callback
                    let has_type_cases = self.check_for_type_assertions(expr);
                    
                    self.tests.push(TypeTest {
                        file: self.file_path.clone(),
                        describe_block: self.get_describe_string(),
                        test_name,
                        line: 0, // Should be calculated
                        has_type_cases,
                        status: if has_type_cases { TestStatus::Passing } else { TestStatus::NoTypeCases },
                    });
                }
            }
        }
        
        walk::walk_call_expression(self, expr);
    }
}

impl TestVisitor {
    fn check_for_type_assertions(&self, _expr: &CallExpression) -> bool {
        // In a real implementation, we'd traverse the callback body.
        // For now, let's keep it as a placeholder.
        false
    }
}
