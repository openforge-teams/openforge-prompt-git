import { create } from "zustand";
import { api } from "./lib/api";
import type {
  AppSettings,
  AppView,
  Folder,
  ModelConfig,
  Project,
  PromptFile,
  PromptVersion,
} from "./types";

interface AppState {
  unlocked: boolean;
  view: AppView;
  theme: string;
  settings: AppSettings | null;
  projects: Project[];
  currentProjectId: number | null;
  folders: Folder[];
  files: PromptFile[];
  currentFileId: number | null;
  versions: PromptVersion[];
  models: ModelConfig[];
  systemPrompt: string;
  userPrompt: string;
  dirty: boolean;
  loading: boolean;
  error: string | null;
  rightTab: "history" | "variables" | "models";

  setView: (view: AppView) => void;
  setRightTab: (tab: "history" | "variables" | "models") => void;
  setError: (error: string | null) => void;
  setUnlocked: (v: boolean) => void;
  setPrompts: (systemPrompt: string, userPrompt: string, dirty?: boolean) => void;
  bootstrap: () => Promise<void>;
  refreshProjects: () => Promise<void>;
  selectProject: (id: number) => Promise<void>;
  selectFile: (id: number) => Promise<void>;
  refreshVersions: () => Promise<void>;
  refreshModels: () => Promise<void>;
  applyTheme: (theme: string) => void;
}

function applyThemeDom(theme: string) {
  const root = document.documentElement;
  if (theme === "system") {
    const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.setAttribute("data-theme", dark ? "dark" : "light");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

export const useAppStore = create<AppState>((set, get) => ({
  unlocked: true,
  view: "editor",
  theme: "system",
  settings: null,
  projects: [],
  currentProjectId: null,
  folders: [],
  files: [],
  currentFileId: null,
  versions: [],
  models: [],
  systemPrompt: "",
  userPrompt: "",
  dirty: false,
  loading: false,
  error: null,
  rightTab: "history",

  setView: (view) => set({ view }),
  setRightTab: (rightTab) => set({ rightTab }),
  setError: (error) => set({ error }),
  setUnlocked: (unlocked) => set({ unlocked }),
  setPrompts: (systemPrompt, userPrompt, dirty = true) =>
    set({ systemPrompt, userPrompt, dirty }),

  applyTheme: (theme) => {
    applyThemeDom(theme);
    set({ theme });
  },

  bootstrap: async () => {
    set({ loading: true, error: null });
    try {
      const settings = await api.getSettings();
      applyThemeDom(settings.theme);
      set({
        settings,
        theme: settings.theme,
        unlocked: !settings.has_app_password,
      });
      if (!settings.has_app_password) {
        await get().refreshProjects();
        await get().refreshModels();
      }
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  refreshProjects: async () => {
    const projects = await api.listProjects();
    set({ projects });
    const current = get().currentProjectId;
    if (!current && projects.length > 0) {
      await get().selectProject(projects[0].id);
    } else if (current) {
      await get().selectProject(current);
    }
  },

  selectProject: async (id: number) => {
    const [folders, files] = await Promise.all([
      api.listFolders(id),
      api.listPromptFiles(id),
    ]);
    set({
      currentProjectId: id,
      folders,
      files,
      currentFileId: null,
      versions: [],
      systemPrompt: "",
      userPrompt: "",
      dirty: false,
    });
    if (files.length > 0) {
      await get().selectFile(files[0].id);
    }
  },

  selectFile: async (id: number) => {
    const file = await api.getPromptFile(id);
    set({
      currentFileId: id,
      systemPrompt: file.system_prompt,
      userPrompt: file.user_prompt,
      dirty: false,
    });
    await get().refreshVersions();
  },

  refreshVersions: async () => {
    const fileId = get().currentFileId;
    if (!fileId) {
      set({ versions: [] });
      return;
    }
    const versions = await api.getVersionHistory(fileId);
    set({ versions });
  },

  refreshModels: async () => {
    const models = await api.listModels();
    set({ models });
  },
}));
