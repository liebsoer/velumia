<script setup lang="ts">
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { Compartment, EditorState, type Extension } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
import { onMounted, onUnmounted, ref, watch } from "vue";
import {
  languageExtension,
  resolveEffectiveSyntax,
  type EffectiveSyntax,
} from "../../lib/prompt-syntax";
import type { PromptContentSyntax } from "../../lib/ipc-types";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    storedSyntax: PromptContentSyntax;
    readOnly?: boolean;
    testId?: string;
  }>(),
  {
    readOnly: false,
    testId: "prompt-content",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const hostRef = ref<HTMLElement | null>(null);
let view: EditorView | null = null;
let suppressUpdate = false;
const languageCompartment = new Compartment();
const readOnlyCompartment = new Compartment();

const editorTheme = EditorView.theme({
  "&": {
    fontSize: "0.8125rem",
    fontFamily:
      'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace',
    backgroundColor: "var(--color-surface)",
    color: "var(--color-text-primary)",
    border: "1px solid var(--color-border)",
    borderRadius: "6px",
  },
  "&.cm-focused": {
    outline: "2px solid var(--color-accent)",
    outlineOffset: "1px",
  },
  ".cm-scroller": {
    minHeight: "12rem",
    maxHeight: "24rem",
    lineHeight: "1.45",
  },
  ".cm-content": {
    padding: "0.625rem 0.75rem",
    caretColor: "var(--color-text-primary)",
  },
  ".cm-gutters": {
    backgroundColor: "var(--color-surface)",
    color: "var(--color-text-muted)",
    borderRight: "1px solid var(--color-border)",
  },
  "&.cm-readonly .cm-cursor": {
    display: "none",
  },
  "&.cm-readonly.cm-focused": {
    outline: "none",
  },
});

function effectiveSyntax(): EffectiveSyntax {
  return resolveEffectiveSyntax(props.storedSyntax, props.modelValue);
}

function buildExtensions(): Extension[] {
  const lang = languageExtension(effectiveSyntax());
  return [
    editorTheme,
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    EditorView.lineWrapping,
    EditorView.updateListener.of((update) => {
      if (update.docChanged && !suppressUpdate) {
        emit("update:modelValue", update.state.doc.toString());
      }
    }),
    languageCompartment.of(lang ? [lang] : []),
    readOnlyCompartment.of([
      EditorState.readOnly.of(props.readOnly),
      EditorView.editable.of(!props.readOnly),
      ...(props.readOnly ? [] : [lineNumbers()]),
    ]),
  ];
}

function createView() {
  if (!hostRef.value) return;
  view?.destroy();
  view = new EditorView({
    parent: hostRef.value,
    state: EditorState.create({
      doc: props.modelValue,
      extensions: buildExtensions(),
    }),
  });
}

function syncDoc(value: string) {
  if (!view) return;
  const current = view.state.doc.toString();
  if (current === value) return;
  suppressUpdate = true;
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: value },
  });
  suppressUpdate = false;
}

function reconfigure() {
  if (!view) return;
  const lang = languageExtension(effectiveSyntax());
  view.dispatch({
    effects: [
      languageCompartment.reconfigure(lang ? [lang] : []),
      readOnlyCompartment.reconfigure([
        EditorState.readOnly.of(props.readOnly),
        EditorView.editable.of(!props.readOnly),
        ...(props.readOnly ? [] : [lineNumbers()]),
      ]),
    ],
  });
}

onMounted(createView);

onUnmounted(() => {
  view?.destroy();
  view = null;
});

watch(
  () => props.modelValue,
  (value) => syncDoc(value),
);

watch(
  () => [props.storedSyntax, props.readOnly] as const,
  () => reconfigure(),
);
</script>

<template>
  <div
    ref="hostRef"
    class="prompt-code-editor"
    :class="{ readonly: readOnly }"
    :data-testid="testId"
  />
</template>

<style scoped>
.prompt-code-editor {
  width: 100%;
}

.prompt-code-editor.readonly :deep(.cm-editor) {
  cursor: default;
}
</style>
