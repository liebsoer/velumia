import type { PromptContentSyntax } from "./ipc-types";

export type EffectiveSyntax = Exclude<PromptContentSyntax, "auto">;

export function detectContentSyntax(content: string): EffectiveSyntax {
  const trimmed = content.trim();
  if (!trimmed) return "plaintext";

  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try {
      JSON.parse(trimmed);
      return "json";
    } catch {
      /* not valid JSON */
    }
  }

  if (
    /^<\?xml[\s>]/.test(trimmed) ||
    /^<[A-Za-z_][\w.-]*(\s|>|\/)/.test(trimmed)
  ) {
    return "xml";
  }

  if (
    /^#{1,6}\s/m.test(trimmed) ||
    /^\s*[-*+]\s/m.test(trimmed) ||
    /\[.+?\]\(.+?\)/.test(trimmed) ||
    /^```/m.test(trimmed) ||
    /^\s*\d+\.\s/m.test(trimmed)
  ) {
    return "markdown";
  }

  return "plaintext";
}
