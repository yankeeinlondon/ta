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
    pub block: String,  // Plain text code block
    #[serde(serialize_with = "span_serializer::serialize")]
    pub span: Span,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct PropertyInfo {
    pub name: String,
    pub type_annotation: Option<String>,
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
            span: Span::new(0, 10),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("TS2322"));
        assert!(json.contains("src/main.ts"));
        assert!(json.contains("\"start\":0"));
        assert!(json.contains("\"end\":10"));
    }
}
