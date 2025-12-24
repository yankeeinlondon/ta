// ============================================================================
// Common Types
// ============================================================================

/**
 * Symbol kinds matching the Rust library's SymbolKind enum
 */
export type SymbolKind =
  | 'Function'
  | 'Class'
  | 'Interface'
  | 'Type'
  | 'Variable'
  | 'Enum';

/**
 * Test status matching the Rust library's TestStatus enum
 */
export type TestStatus =
  | 'Passing'
  | 'Failing'
  | 'NoTypeCases';

/**
 * Scope type where a type error occurred
 */
export type ScopeType = 'Function' | 'Method' | 'TypeUtility' | 'ModuleLevel';

/**
 * Scope of a dependency
 */
export type DependencyScope = 'Local' | 'Repo' | 'Module' | 'External';

// ============================================================================
// Source Command - Type Error Analysis
// ============================================================================

/**
 * A single type error detected in TypeScript source
 */
export interface TypeError {
  /** Error code (e.g., "TS2322") or "error" fallback */
  id: string;
  /** Human-readable error message */
  message: string;
  /** Relative file path where error occurred */
  file: string;
  /** Line number (1-indexed) */
  line: number;
  /** Column number (1-indexed) */
  column: number;
  /** Scope identifier (e.g., "myFunction", "MyClass::method", "global") */
  scope: string;
  /** Legacy plain text code block (deprecated, use source_code instead) */
  block: string;
  /** Context-aware code extraction (optional) */
  source_code?: {
    /** Complete code of the enclosing scope */
    full_code: string;
    /** Truncated code snippet showing the error */
    display_code: string;
    /** Type of scope containing the error */
    scope_type: ScopeType;
    /** Name of the scope (function/method/class name) */
    scope_name: string;
  };
  /** Source span information */
  span: {
    /** Byte offset of error start */
    start: number;
    /** Byte offset of error end */
    end: number;
  };
}

/**
 * JSON output from `ta source --json`
 */
export type SourceOutput = TypeError[];

// ============================================================================
// Symbols Command - Symbol Extraction
// ============================================================================

/**
 * Function or method parameter
 */
export interface ParameterInfo {
  /** Parameter name */
  name: string;
  /** TypeScript type annotation (if present) */
  type_annotation: string | null;
  /** JSDoc description (if present) */
  description: string | null;
}

/**
 * Class or interface property
 */
export interface PropertyInfo {
  /** Property name */
  name: string;
  /** TypeScript type annotation (if present) */
  type_annotation: string | null;
  /** JSDoc description (if present) */
  description: string | null;
}

/**
 * A symbol (function, class, interface, etc.)
 */
export interface SymbolInfo {
  /** Symbol name */
  name: string;
  /** Type of symbol */
  kind: SymbolKind;
  /** Relative file path */
  file: string;
  /** Starting line number (1-indexed) */
  start_line: number;
  /** Ending line number (1-indexed) */
  end_line: number;
  /** Whether symbol is exported */
  exported: boolean;
  /** Parameters (for functions/methods) */
  parameters: ParameterInfo[] | null;
  /** Properties (for classes/interfaces) */
  properties: PropertyInfo[] | null;
  /** Return type annotation (for functions) */
  return_type: string | null;
  /** JSDoc comment */
  jsdoc: string | null;
}

/**
 * JSON output from `ta symbols --json`
 */
export type SymbolsOutput = SymbolInfo[];

// ============================================================================
// File Command - File-Level Dependencies
// ============================================================================

/**
 * Import statement information
 */
export interface ImportInfo {
  /** Source module path (e.g., "./utils", "react") */
  source: string;
  /** Imported symbol names */
  symbols: string[];
}

/**
 * File-level dependency information
 */
export interface FileDependencies {
  /** Relative file path */
  file: string;
  /** Import statements in the file */
  imports: ImportInfo[];
}

/**
 * JSON output from `ta file --json`
 */
export type FileOutput = FileDependencies[];

// ============================================================================
// Deps Command - Symbol-Level Dependencies
// ============================================================================

/**
 * A single symbol dependency
 */
export interface SymbolDependency {
  /** Name of the dependency */
  name: string;
  /** Scope classification */
  scope: DependencyScope;
  /** Source file (if known) */
  file: string | null;
}

/**
 * Symbol-level dependency information
 */
export interface SymbolDependencies {
  /** Symbol name */
  symbol: string;
  /** File containing the symbol */
  file: string;
  /** List of dependencies */
  dependencies: SymbolDependency[];
}

/**
 * JSON output from `ta deps --json`
 */
export type DepsOutput = SymbolDependencies[];

// ============================================================================
// Test Command - Type Test Detection
// ============================================================================

/**
 * A detected type test
 */
export interface TypeTest {
  /** Relative file path */
  file: string;
  /** Name of the describe() block */
  describe_block: string;
  /** Name of the test case */
  test_name: string;
  /** Line number (1-indexed) */
  line: number;
  /** Whether test has type cases array */
  has_type_cases: boolean;
  /** Test status */
  status: TestStatus;
}

/**
 * JSON output from `ta test --json`
 */
export type TestOutput = TypeTest[];

// ============================================================================
// Watch Command - File System Events
// ============================================================================

/**
 * Base structure for all watch events
 */
export interface BaseEvent<T extends string, D> {
  type: T;
  data: D;
}

export type WatchEvent =
  | BaseEvent<'SourceFileChanged', { file: string; content: string }>
  | BaseEvent<'SourceFileCreated', { file: string }>
  | BaseEvent<'SourceFileRemoved', { file: string }>
  | BaseEvent<'SymbolRenamed', { old_name: string; new_name: string; file: string }>
  | BaseEvent<'SymbolAdded', { name: string; kind: SymbolKind; file: string }>
  | BaseEvent<'SymbolRemoved', { name: string; file: string }>
  | BaseEvent<'ModuleDepChanged', { file: string }>
  | BaseEvent<'ExternalDepChanged', { package: string }>
  | BaseEvent<'TestStatusChanged', { file: string; test: string; status: TestStatus }>
  | BaseEvent<'NewFailingTest', { file: string; test: string }>
  | BaseEvent<'TestFixed', { file: string; test: string }>
  | BaseEvent<'NewTestAdded', { file: string; test: string }>;

/**
 * A handler function that receives a watch event
 */
export type WatchHandler = (event: WatchEvent) => void | Promise<void>;

/**
 * Specific handler types for individual events
 */
export type SymbolAddedHandler = (event: BaseEvent<'SymbolAdded', { name: string; kind: SymbolKind; file: string }>) => void | Promise<void>;
export type TestStatusChangedHandler = (event: BaseEvent<'TestStatusChanged', { file: string; test: string; status: TestStatus }>) => void | Promise<void>;
