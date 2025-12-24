use oxc_span::Span;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TypeError {
    pub id: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub scope: String,  // "file::symbol" format
    pub block: String,  // Plain text code block (legacy - kept for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<SourceCode>,  // New field - context-aware code extraction
    #[serde(serialize_with = "span_serializer::serialize")]
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceCode {
    pub full_code: String,
    pub display_code: String,
    pub scope_type: crate::highlighting::ScopeType,
    pub scope_name: String,
}

mod span_serializer {
    use oxc_span::Span;
    use serde::Serializer;

    pub fn serialize<S>(span: &Span, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Span", 2)?;
        state.serialize_field("start", &span.start)?;
        state.serialize_field("end", &span.end)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub exported: bool,
    pub parameters: Option<Vec<ParameterInfo>>,
    pub properties: Option<Vec<PropertyInfo>>,
    pub return_type: Option<String>,
    pub jsdoc: Option<String>,
}

impl SymbolInfo {
    /// Create a compact string representation of the symbol
    /// Examples:
    /// - function createContext(name: string): Context
    /// - class UserApi
    /// - interface User { id: number, name: string }
    /// - type ApiResponse<T>
    pub fn display_signature(&self) -> String {
        match self.kind {
            SymbolKind::Function => {
                let params = if let Some(params) = &self.parameters {
                    params.iter()
                        .map(|p| {
                            if let Some(ty) = &p.type_annotation {
                                format!("{}: {}", p.name, ty)
                            } else {
                                p.name.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    String::new()
                };

                if let Some(ret) = &self.return_type {
                    format!("function {}({}): {}", self.name, params, ret)
                } else {
                    format!("function {}({})", self.name, params)
                }
            }
            SymbolKind::Class => {
                format!("class {}", self.name)
            }
            SymbolKind::Interface => {
                if let Some(props) = &self.properties {
                    if props.is_empty() {
                        format!("interface {}", self.name)
                    } else {
                        let prop_str = props.iter()
                            .take(3) // Limit to first 3 properties
                            .map(|p| {
                                if let Some(ty) = &p.type_annotation {
                                    format!("{}: {}", p.name, ty)
                                } else {
                                    p.name.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let suffix = if props.len() > 3 { ", ..." } else { "" };
                        format!("interface {} {{ {}{} }}", self.name, prop_str, suffix)
                    }
                } else {
                    format!("interface {}", self.name)
                }
            }
            SymbolKind::Type => {
                format!("type {}", self.name)
            }
            SymbolKind::Variable => {
                format!("variable {}", self.name)
            }
            SymbolKind::Enum => {
                format!("enum {}", self.name)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Enum,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PropertyInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileDependencies {
    pub file: String,
    pub repo_dependencies: Vec<String>,
    pub external_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolDependencies {
    pub symbol: String,
    pub file: String,
    pub dependencies: Vec<SymbolDependency>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolDependency {
    pub name: String,
    pub scope: DependencyScope,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DependencyScope {
    Local,
    Repo,
    Module,
    External,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeTest {
    pub file: String,
    pub describe_block: String,
    pub test_name: String,
    pub line: usize,
    pub has_type_cases: bool,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum TestStatus {
    Passing,
    Failing,
    NoTypeCases,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_serialization() {
        let error = TypeError {
            id: "TS2322".to_string(),
            message: "Type 'string' is not assignable to type 'number'".to_string(),
            file: "src/main.ts".to_string(),
            line: 10,
            column: 5,
            scope: "main".to_string(),
            block: "const x: number = 'hello';".to_string(),
            source_code: None,
            span: Span::new(0, 10),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("TS2322"));
        assert!(json.contains("src/main.ts"));
        assert!(json.contains("\"start\":0"));
        assert!(json.contains("\"end\":10"));
        // source_code is None, should be skipped in serialization
        assert!(!json.contains("source_code"));
    }
}
