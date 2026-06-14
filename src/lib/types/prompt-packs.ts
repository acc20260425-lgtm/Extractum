export interface PromptPackLibrary {
  packs: PromptPack[];
}

export interface PromptPack {
  packId: string;
  displayName: string;
  activeVersion: PromptPackVersion | null;
}

export interface PromptPackVersion {
  packVersionId: number;
  packVersion: string;
  schemaVersion: string;
  lifecycleStatus: string;
  defaultControlPreset: string;
  defaultEvidenceMode: string;
  defaultIncludeComments: boolean;
  stages: PromptPackStageTemplate[];
  schemaAssets: PromptPackSchemaAsset[];
}

export interface PromptPackStageTemplate {
  stageName: string;
  stageOrder: number;
  providerFamily: string;
  inputSchemaId: string;
  outputSchemaId: string;
}

export interface PromptPackSchemaAsset {
  schemaId: string;
  schemaKind: string;
  contentHash: string;
}
