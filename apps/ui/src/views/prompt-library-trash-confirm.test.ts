import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PromptSummary } from "../lib/api";
import PromptLibraryView from "./PromptLibraryView.vue";

const { trashPrompt, listPrompts, listPromptFolders, listTags } = vi.hoisted(() => ({
  trashPrompt: vi.fn(),
  listPrompts: vi.fn(),
  listPromptFolders: vi.fn(),
  listTags: vi.fn(),
}));

const samplePrompt: PromptSummary = {
  id: "p-trash-confirm",
  title: "Trash confirm",
  slug: "trash-confirm",
  folder_id: null,
  tags: [],
  is_favorite: false,
  content_syntax: "plaintext",
  updated_at: "2026-01-01T00:00:00Z",
  lifecycle_status: "active",
};

vi.mock("../lib/api", () => ({
  api: {
    listPrompts,
    listPromptFolders,
    listTags,
    trashPrompt,
    createPrompt: vi.fn(),
    createPromptFolder: vi.fn(),
    movePromptToFolder: vi.fn(),
    setPromptFavorite: vi.fn(),
    unsetPromptFavorite: vi.fn(),
    archivePrompt: vi.fn(),
    unarchivePrompt: vi.fn(),
    restorePrompt: vi.fn(),
    getPrompt: vi.fn(),
    savePromptContent: vi.fn(),
    updatePrompt: vi.fn(),
    listPromptVersions: vi.fn(),
    getPromptVersionContent: vi.fn(),
    restorePromptVersion: vi.fn(),
    checkAuthorize: vi.fn().mockResolvedValue({ allowed: true }),
  },
}));

async function openPromptInTree(wrapper: ReturnType<typeof mount>) {
  const allRow = wrapper
    .findAll(".tree-row")
    .find((r) => r.text().includes("All prompts"));
  expect(allRow).toBeDefined();
  await allRow!.trigger("click");
  await flushPromises();

  const unfiledBtn = wrapper
    .findAll("button.folder-child-row")
    .find((b) => b.text().includes("Unfiled"));
  expect(unfiledBtn).toBeDefined();
  await unfiledBtn!.trigger("click");
  await flushPromises();

  const promptBtn = wrapper.find('[data-testid="folder-child-prompt"]');
  expect(promptBtn.exists()).toBe(true);
  await promptBtn.trigger("click");
  await flushPromises();
}

describe("PromptLibraryView trash confirm (X-02)", () => {
  beforeEach(() => {
    trashPrompt.mockReset();
    listPrompts.mockResolvedValue([samplePrompt]);
    listPromptFolders.mockResolvedValue([]);
    listTags.mockResolvedValue([]);
  });

  it("x_02_trash_requires_confirmation", async () => {
    const wrapper = mount(PromptLibraryView, {
      global: {
        stubs: {
          PromptDetailPanel: {
            template:
              '<button data-testid="initiate-trash" @click="$emit(\'openDelete\')">Move to trash</button>',
            props: ["prompt", "tags", "locationBreadcrumb"],
            emits: [
              "openDelete",
              "refresh",
              "error",
              "openMove",
              "favoriteToggle",
              "openSettings",
            ],
          },
        },
      },
    });
    await flushPromises();

    await openPromptInTree(wrapper);
    expect(wrapper.find('[data-testid="trash-confirm-dialog"]').exists()).toBe(false);

    await wrapper.find('[data-testid="initiate-trash"]').trigger("click");
    expect(wrapper.find('[data-testid="trash-confirm-dialog"]').exists()).toBe(true);
    expect(trashPrompt).not.toHaveBeenCalled();

    await wrapper.find('[data-testid="cancel-trash"]').trigger("click");
    expect(wrapper.find('[data-testid="trash-confirm-dialog"]').exists()).toBe(false);
    expect(trashPrompt).not.toHaveBeenCalled();
    expect(listPrompts).toHaveBeenCalled();
  });
});
