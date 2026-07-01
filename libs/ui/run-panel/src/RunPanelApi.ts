import type {
  RunChunkPayload,
  RunErrorPayload,
  RunEventName,
  RunSessionPayload,
  SendMessageInput,
  SessionSummary,
  StartRunInput,
  StartRunResult,
  StopRunInput,
  TranscriptLine,
} from "./types";

export interface RunPanelApi {
  canRun(): Promise<boolean>;
  listSessions(entityId: string): Promise<SessionSummary[]>;
  getSessionTranscript(entityId: string, sessionId: string): Promise<TranscriptLine[]>;
  startRun(input: StartRunInput): Promise<StartRunResult>;
  sendMessage(input: SendMessageInput): Promise<StartRunResult>;
  stopRun(input: StopRunInput): Promise<void>;
  deleteSession(entityId: string, sessionId: string): Promise<void>;
  onRunEvent(event: RunEventName, handler: (payload: unknown) => void): () => void;
}

export type {
  RunChunkPayload,
  RunErrorPayload,
  RunSessionPayload,
  SendMessageInput,
  SessionSummary,
  StartRunInput,
  StartRunResult,
  StopRunInput,
  TranscriptLine,
};
