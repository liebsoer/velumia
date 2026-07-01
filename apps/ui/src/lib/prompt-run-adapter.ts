import type { RunPanelApi } from "@velumia/ui-run-panel";
import { api, onPromptRunEvent } from "./api";

export function createPromptRunApi(): RunPanelApi {
  return {
    canRun: () => api.canRunPrompt(),

    async listSessions(entityId) {
      const sessions = await api.listPromptSessions(entityId);
      return sessions.map(({ id, created_at, updated_at, stopped }) => ({
        id,
        created_at,
        updated_at,
        stopped,
      }));
    },

    getSessionTranscript: (entityId, sessionId) =>
      api.getSessionTranscript(entityId, sessionId),

    startRun: ({ entityId, userMessage, variables, allowEmptyVariables }) =>
      api.startPromptRun({
        prompt_id: entityId,
        user_message: userMessage,
        variables,
        allow_empty_variables: allowEmptyVariables,
      }),

    sendMessage: ({ entityId, session_id, user_message }) =>
      api.sendPromptMessage({
        prompt_id: entityId,
        session_id,
        user_message,
      }),

    stopRun: ({ entityId, session_id, run_id }) =>
      api.stopPromptRun({
        prompt_id: entityId,
        session_id,
        run_id,
      }),

    deleteSession: (entityId, sessionId) =>
      api.deletePromptSession(entityId, sessionId),

    onRunEvent(event, handler) {
      return onPromptRunEvent(event, handler);
    },
  };
}

export const promptRunApi = createPromptRunApi();
