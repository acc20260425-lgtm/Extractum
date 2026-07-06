export interface ProjectAddSourceContext {
  projectId: number;
  connectedSourceIds: Set<number>;
  onConnectExistingSource(sourceId: number): void | Promise<void>;
  onConnectAddedSources(sourceIds: number[]): void | Promise<void>;
}
