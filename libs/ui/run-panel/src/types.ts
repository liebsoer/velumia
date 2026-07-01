export interface SessionSummary {
  id: string;
  created_at: string;
  updated_at: string;
  stopped: boolean;
}

export type TranscriptLine =
  | { type: "run_config"; instructions: string; model: string }
  | { type: "message"; role: string; content: string }
  | { type: "meta"; event: string };

export interface StartRunInput {
  entityId: string;
  userMessage?: string;
  variables?: Record<string, string>;
  allowEmptyVariables?: boolean;
}

export interface StartRunResult {
  session_id: string;
  run_id: string;
}

export interface SendMessageInput {
  entityId: string;
  session_id: string;
  user_message: string;
}

export interface StopRunInput {
  entityId: string;
  session_id: string;
  run_id: string;
}

export interface RunChunkPayload {
  session_id: string;
  run_id: string;
  chunk: string;
  done: boolean;
}

export interface RunSessionPayload {
  session_id: string;
  run_id: string;
}

export interface RunErrorPayload {
  session_id: string;
  run_id: string;
  message: string;
}

export const RUN_EVENTS = {
  chunk: "prompt-run-chunk",
  done: "prompt-run-done",
  error: "prompt-run-error",
  stopped: "prompt-run-stopped",
} as const;

export type RunEventName = (typeof RUN_EVENTS)[keyof typeof RUN_EVENTS];
