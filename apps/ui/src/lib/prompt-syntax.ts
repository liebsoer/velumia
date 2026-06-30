import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";
import { xml } from "@codemirror/lang-xml";
import type { Extension } from "@codemirror/state";
import type { PromptContentSyntax } from "./ipc-types";
import { detectContentSyntax } from "./detect-content-syntax";

export type EffectiveSyntax = Exclude<PromptContentSyntax, "auto">;

export const PROMPT_SYNTAX_OPTIONS: { value: PromptContentSyntax; label: string }[] = [
  { value: "auto", label: "Auto" },
  { value: "plaintext", label: "Plain text" },
  { value: "markdown", label: "Markdown" },
  { value: "xml", label: "XML" },
  { value: "json", label: "JSON" },
];

export function resolveEffectiveSyntax(
  stored: PromptContentSyntax,
  content: string,
): EffectiveSyntax {
  if (stored !== "auto") return stored;
  return detectContentSyntax(content);
}

export function languageExtension(syntax: EffectiveSyntax): Extension | null {
  switch (syntax) {
    case "markdown":
      return markdown();
    case "xml":
      return xml();
    case "json":
      return json();
    default:
      return null;
  }
}
