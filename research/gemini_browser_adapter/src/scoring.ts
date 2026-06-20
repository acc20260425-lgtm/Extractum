export type EditableCandidateScoreInput = {
  aria: string;
  topRatio: number;
  width: number;
  height: number;
  visible: boolean;
  editable: boolean;
};

export type ButtonCandidateScoreInput = {
  label: string;
  topRatio: number;
  rightRatio: number;
  width: number;
  height: number;
  visible: boolean;
  enabled: boolean;
};

export function scoreEditableCandidate(input: EditableCandidateScoreInput): number {
  if (!input.visible || !input.editable) return 0;
  let score = 0;
  if (input.width >= 300) score += 2;
  if (input.height >= 24 && input.height <= 240) score += 2;
  if (input.topRatio >= 0.45) score += 2;
  if (/ask|message|prompt|gemini|enter|type/i.test(input.aria)) score += 3;
  if (input.editable) score += 1;
  return score;
}

export function scoreButtonCandidate(input: ButtonCandidateScoreInput): number {
  if (!input.visible || !input.enabled) return 0;
  let score = 0;
  if (input.width >= 24 && input.height >= 24) score += 2;
  if (/send|submit|run|arrow|message/i.test(input.label)) score += 5;
  if (input.topRatio >= 0.45) score += 1;
  if (input.rightRatio >= 0.55) score += 2;
  return score;
}
