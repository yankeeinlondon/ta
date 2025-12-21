use std::collections::HashSet;
use crate::models::TypeError;
use oxc_ast::visit::{walk, Visit};
use oxc_ast::ast::*;
use oxc_semantic::{Semantic, ScopeFlags};
use oxc_span::{Span, GetSpan};
use oxc_diagnostics::OxcDiagnostic;
use miette::SourceSpan;

pub struct TypeErrorVisitor<'a> {
    pub errors: Vec<TypeError>, // Output
    pub source: &'a str,
    pub semantic: &'a Semantic<'a>,
    pub diagnostics: &'a Vec<OxcDiagnostic>, // Input
    current_scope: Vec<String>,
    processed_errors: HashSet<usize>,
}

impl<'a> TypeErrorVisitor<'a> {
    pub fn new(source: &'a str, semantic: &'a Semantic<'a>, diagnostics: &'a Vec<OxcDiagnostic>) -> Self {
        Self {
            errors: Vec::new(),
            source,
            semantic,
            diagnostics,
            current_scope: Vec::new(),
            processed_errors: HashSet::new(),
        }
    }

    fn get_scope_string(&self) -> String {
        if self.current_scope.is_empty() {
            return "global".to_string();
        }
        self.current_scope.join("::")
    }

    fn to_oxc_span(span: &SourceSpan) -> Span {
        let start = span.offset() as u32;
        let end = (span.offset() + span.len()) as u32;
        Span::new(start, end)
    }

    fn add_error(&mut self, index: usize, error: &OxcDiagnostic, span: Span) {
        if self.processed_errors.contains(&index) {
            return;
        }
        
        let error_span = error.labels.as_ref()
            .and_then(|labels| labels.first())
            .map(|l| Self::to_oxc_span(l.inner()))
            .unwrap_or(span);
            
        let message = error.to_string();
        
        let (line, column) = self.get_line_col(error_span.start);
        
        let block = self.source.get(error_span.start as usize..error_span.end as usize)
            .unwrap_or("").to_string();

        self.errors.push(TypeError {
            id: "error".to_string(),
            message,
            file: "unknown".to_string(),
            line,
            column,
            scope: self.get_scope_string(),
            block,
            span: error_span,
        });
        
        self.processed_errors.insert(index);
    }

    fn get_line_col(&self, offset: u32) -> (usize, usize) {
        let offset = offset as usize;
        if offset >= self.source.len() {
            return (0, 0);
        }
        let before = &self.source[..offset];
        let line = before.lines().count();
        let last_line = before.lines().last().unwrap_or("");
        let column = last_line.chars().count(); 
        (line, column)
    }

    fn check_errors_in_span(&mut self, span: Span) {
        for (i, error) in self.diagnostics.iter().enumerate() {
             if !self.processed_errors.contains(&i) {
                let error_span = error.labels.as_ref()
                    .and_then(|labels| labels.first())
                    .map(|l| Self::to_oxc_span(l.inner()))
                    .unwrap_or(Span::default());
                    
                if span.contains_inclusive(error_span) {
                    self.add_error(i, error, error_span);
                }
             }
        }
    }
}

impl<'a> Visit<'a> for TypeErrorVisitor<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        walk::walk_program(self, program);
        
        // Capture any remaining errors at global scope
        for (i, error) in self.diagnostics.iter().enumerate() {
            if !self.processed_errors.contains(&i) {
                 let span = error.labels.as_ref()
                    .and_then(|labels| labels.first())
                    .map(|l| Self::to_oxc_span(l.inner()))
                    .unwrap_or(Span::default());
                 self.add_error(i, error, span);
            }
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        let mut pushed = false;
        if let Some(id) = &func.id {
            self.current_scope.push(id.name.to_string());
            pushed = true;
        }
        
        walk::walk_function(self, func, flags);
        
        self.check_errors_in_span(func.span);

        if pushed {
            self.current_scope.pop();
        }
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let mut pushed = false;
        if let Some(id) = &class.id {
             self.current_scope.push(id.name.to_string());
             pushed = true;
        } else {
             // For anonymous classes (e.g. export default class {}), maybe push "anonymous"? 
             // Or keep parent scope.
             // Let's keep parent scope for consistency with function logic change.
        }
        
        walk::walk_class(self, class);
        
        self.check_errors_in_span(class.span);
        
        if pushed {
            self.current_scope.pop();
        }
    }
    
    fn visit_method_definition(&mut self, def: &MethodDefinition<'a>) {
        let name = match &def.key {
            PropertyKey::StaticIdentifier(id) => id.name.to_string(),
            PropertyKey::PrivateIdentifier(id) => id.name.to_string(),
            _ => "dynamic_method".to_string(),
        };
        
        self.current_scope.push(name);
        walk::walk_method_definition(self, def);
        self.current_scope.pop();
    }
    
    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        walk::walk_statement(self, stmt);
        self.check_errors_in_span(stmt.span());
    }
    
    fn visit_ts_type_annotation(&mut self, annotation: &TSTypeAnnotation<'a>) {
         walk::walk_ts_type_annotation(self, annotation);
         self.check_errors_in_span(annotation.span);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    fn parse_and_visit(source: &str) -> Vec<TypeError> {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        // Pass source text to SemanticBuilder
        let semantic_ret = SemanticBuilder::new(source).build(&ret.program);
        let semantic = semantic_ret.semantic;
        let diagnostics = semantic_ret.errors;
        
        let mut visitor = TypeErrorVisitor::new(source, &semantic, &diagnostics);
        visitor.visit_program(&ret.program);
        
        visitor.errors
    }

    #[test]
    fn test_redeclaration_error() {
        let source = "let x = 1; let x = 2;";
        let errors = parse_and_visit(source);
        // Note: OXC might return 1 or 2 errors (redeclaration).
        assert!(!errors.is_empty());
        assert!(errors[0].scope.contains("global"));
    }

    #[test]
    fn test_function_scope_error() {
        let source = "function foo() { let y = 1; let y = 2; }";
        let errors = parse_and_visit(source);
        assert!(!errors.is_empty());
        assert!(errors[0].scope.contains("foo"));
    }

    #[test]
    fn test_class_scope_error() {
        let source = "class MyClass { method() { let z = 1; let z = 2; } }";
        let errors = parse_and_visit(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].scope, "MyClass::method");
    }

    #[test]
    fn test_nested_function_scope() {
        let source = "function outer() { function inner() { let a = 1; let a = 2; } }";
        let errors = parse_and_visit(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].scope, "outer::inner");
    }

    #[test]
    fn test_no_errors() {
        let source = "let x = 1;";
        let errors = parse_and_visit(source);
        assert!(errors.is_empty());
    }
}
