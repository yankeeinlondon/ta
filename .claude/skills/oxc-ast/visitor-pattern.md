# Visitor Pattern for OXC

The Visitor pattern is the primary way to traverse the AST and extract information. OXC provides a `Visit` trait that fires callbacks when specific node types are encountered.

## Basic Visitor Structure

```rust
use oxc::ast::visit::{walk, Visit};
use oxc::ast::ast::TSInterfaceDeclaration;

struct MyVisitor {
    // Your state goes here
    interface_names: Vec<String>,
}

impl<'a> Visit<'a> for MyVisitor {
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let name = it.id.name.as_str();
        self.interface_names.push(name.to_string());

        // IMPORTANT: Continue walking children
        walk::walk_ts_interface_declaration(self, it);
    }
}

// Usage
let mut visitor = MyVisitor { interface_names: vec![] };
visitor.visit_program(&ret.program);
```

## Example: Interface Inheritance Graph

This example builds a dependency graph showing which interfaces extend which others.

```rust
use oxc::allocator::Allocator;
use oxc::ast::ast::{TSInterfaceDeclaration, Expression};
use oxc::ast::visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::dot::{Dot, Config};
use std::collections::HashMap;

struct TypeGraph {
    graph: DiGraph<String, ()>,
    indices: HashMap<String, NodeIndex>,
}

impl TypeGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            indices: HashMap::new(),
        }
    }

    fn add_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.indices.get(name) {
            return idx;
        }
        let idx = self.graph.add_node(name.to_string());
        self.indices.insert(name.to_string(), idx);
        idx
    }

    fn add_dependency(&mut self, from: &str, to: &str) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);
        self.graph.add_edge(from_idx, to_idx, ());
    }
}

struct InterfaceVisitor<'a> {
    graph: TypeGraph,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> InterfaceVisitor<'a> {
    fn new() -> Self {
        Self {
            graph: TypeGraph::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a> Visit<'a> for InterfaceVisitor<'a> {
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let interface_name = it.id.name.as_str();

        // Ensure the interface exists in the graph
        self.graph.add_node(interface_name);

        // Check if it extends other interfaces
        if let Some(extends) = &it.extends {
            for heritage in extends {
                // Extract the parent interface name
                if let Expression::IdentifierReference(ident) = &heritage.expression {
                    let parent_name = ident.name.as_str();
                    println!("{} extends {}", interface_name, parent_name);
                    self.graph.add_dependency(interface_name, parent_name);
                }
            }
        }

        // Continue walking children
        walk::walk_ts_interface_declaration(self, it);
    }
}

fn main() {
    let source_code = r#"
        interface Entity {
            id: string;
        }

        interface User extends Entity {
            username: string;
        }

        interface Admin extends User {
            permissions: string[];
        }
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, source_code, source_type).parse();

    let mut visitor = InterfaceVisitor::new();
    visitor.visit_program(&ret.program);

    // Output DOT format for Graphviz
    println!("{:?}", Dot::with_config(&visitor.graph.graph, &[Config::EdgeNoLabel]));
}
```

## Extracting Interface Fields

To extract the **shape** of an interface (property names and types):

```rust
impl<'a> Visit<'a> for InterfaceVisitor<'a> {
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        println!("Interface: {}", it.id.name);

        for signature in &it.body.body {
            // Only care about property signatures (e.g., "name: string")
            if let oxc::ast::ast::TSSignature::TSPropertySignature(prop) = signature {
                // Get the key name
                let key = match &prop.key {
                    oxc::ast::ast::PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    _ => "computed_key",
                };

                // Get the value type
                let type_name = if let Some(ann) = &prop.type_annotation {
                    match &ann.type_annotation {
                        oxc::ast::ast::TSType::TSStringKeyword(_) => "string",
                        oxc::ast::ast::TSType::TSNumberKeyword(_) => "number",
                        oxc::ast::ast::TSType::TSTypeReference(_) => "reference",
                        _ => "complex",
                    }
                } else {
                    "any"
                };

                println!("  - {}: {}", key, type_name);
            }
        }

        walk::walk_ts_interface_declaration(self, it);
    }
}
```

## Common Visitor Methods

| Method | Fires On | Use Case |
|--------|----------|----------|
| `visit_function` | Functions (declarations & expressions) | Finding functions |
| `visit_arrow_function_expression` | Arrow functions `() => {}` | Finding arrow functions |
| `visit_method_definition` | Class methods | Finding methods in classes |
| `visit_ts_interface_declaration` | TypeScript interfaces | Type extraction |
| `visit_variable_declarator` | Variable declarations | Finding variables |
| `visit_call_expression` | Function calls `foo()` | Finding usages |
| `visit_import_declaration` | Import statements | Dependency extraction |

## Important: Always Walk Children

**Always call `walk::walk_*` at the end of your visitor method to continue traversal:**

```rust
fn visit_function(&mut self, func: &Function<'a>) {
    // Your logic here

    // REQUIRED: Continue walking nested nodes
    walk::walk_function(self, func);
}
```

**If you forget this, nested nodes won't be visited!**

## When to Use Visitor vs Semantic Iteration

**Use Visitor when:**
- You need to process nodes in a specific order (top-down)
- You want to maintain parent-child context during traversal
- You're building complex data structures (like graphs)

**Use Semantic iteration when:**
- You need symbol/reference information
- Order doesn't matter
- You want maximum performance (flat array iteration)

```rust
// Semantic iteration (faster for simple cases)
for node in semantic.nodes().iter() {
    if let AstKind::Function(func) = node.kind() {
        // Process function
    }
}
```
