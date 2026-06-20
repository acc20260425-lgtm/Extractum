export interface GeminiSelectorCandidate {
  selector: string;
  score: number;
  purpose: "composer" | "send" | "answer" | "manual_action";
}

export const GEMINI_DOM_CONTRACT_VERSION = "2026-06-20-resilient-scoring";

export const composerCandidates: GeminiSelectorCandidate[] = [
  { selector: "rich-textarea textarea", score: 100, purpose: "composer" },
  { selector: "textarea[aria-label*='prompt' i]", score: 80, purpose: "composer" },
  { selector: "[contenteditable='true']", score: 50, purpose: "composer" },
];

export const sendCandidates: GeminiSelectorCandidate[] = [
  { selector: "button[aria-label*='send' i]", score: 100, purpose: "send" },
  { selector: "button[type='submit']", score: 70, purpose: "send" },
];

export const answerCandidates: GeminiSelectorCandidate[] = [
  { selector: "[data-response-index]", score: 100, purpose: "answer" },
  { selector: "message-content", score: 90, purpose: "answer" },
  { selector: "article [dir='ltr']", score: 65, purpose: "answer" },
];
