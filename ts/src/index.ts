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
