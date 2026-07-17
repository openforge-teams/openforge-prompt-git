import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BatchTask,
  CompareResult,
  CompareTask,
  ExportCodeRequest,
  Folder,
  InferenceParams,
  ModelConfig,
  ModelConfigInput,
  Project,
  PromptFile,
  PromptVersion,
  ScoreInput,
  VariableCase,
  VersionDiff,
  VersionTag,
} from "../types";

export const api = {
  createProject: (name: string, description?: string) =>
    invoke<Project>("create_project", { name, description }),
  listProjects: () => invoke<Project[]>("list_projects"),
  updateProject: (id: number, name: string, description?: string) =>
    invoke<Project>("update_project", { id, name, description }),
  deleteProject: (id: number) => invoke<void>("delete_project", { id }),

  createFolder: (projectId: number, name: string, parentId?: number | null) =>
    invoke<Folder>("create_folder", {
      projectId,
      parentId: parentId ?? null,
      name,
    }),
  listFolders: (projectId: number) =>
    invoke<Folder[]>("list_folders", { projectId }),
  deleteFolder: (id: number) => invoke<void>("delete_folder", { id }),

  createPromptFile: (
    projectId: number,
    name: string,
    folderId?: number | null,
    systemPrompt?: string,
    userPrompt?: string,
  ) =>
    invoke<PromptFile>("create_prompt_file", {
      projectId,
      folderId: folderId ?? null,
      name,
      systemPrompt,
      userPrompt,
    }),
  listPromptFiles: (projectId: number) =>
    invoke<PromptFile[]>("list_prompt_files", { projectId }),
  getPromptFile: (id: number) => invoke<PromptFile>("get_prompt_file", { id }),
  renamePromptFile: (id: number, name: string) =>
    invoke<PromptFile>("rename_prompt_file", { id, name }),
  deletePromptFile: (id: number) => invoke<void>("delete_prompt_file", { id }),

  commitPromptVersion: (payload: {
    fileId: number;
    systemPrompt: string;
    userPrompt: string;
    commitMessage: string;
    remark?: string;
    tagIds?: number[];
  }) =>
    invoke<PromptVersion>("commit_prompt_version", {
      fileId: payload.fileId,
      systemPrompt: payload.systemPrompt,
      userPrompt: payload.userPrompt,
      commitMessage: payload.commitMessage,
      remark: payload.remark,
      tagIds: payload.tagIds,
    }),
  getVersionHistory: (fileId: number) =>
    invoke<PromptVersion[]>("get_version_history", { fileId }),
  getVersion: (versionHash: string) =>
    invoke<PromptVersion>("get_version", { versionHash }),
  diffVersions: (versionHash1: string, versionHash2: string) =>
    invoke<VersionDiff>("diff_versions", { versionHash1, versionHash2 }),
  rollbackVersion: (fileId: number, targetVersionHash: string) =>
    invoke<PromptVersion>("rollback_version", { fileId, targetVersionHash }),
  updateVersionRemark: (versionHash: string, remark: string) =>
    invoke<PromptVersion>("update_version_remark", { versionHash, remark }),

  listTags: (projectId: number) =>
    invoke<VersionTag[]>("list_tags", { projectId }),
  createTag: (projectId: number, name: string, color: string) =>
    invoke<VersionTag>("create_tag", { projectId, name, color }),
  deleteTag: (tagId: number) => invoke<void>("delete_tag", { tagId }),
  attachTag: (versionHash: string, tagId: number) =>
    invoke<void>("attach_tag", { versionHash, tagId }),
  detachTag: (versionHash: string, tagId: number) =>
    invoke<void>("detach_tag", { versionHash, tagId }),
  filterHistoryByTag: (fileId: number, tagId: number) =>
    invoke<PromptVersion[]>("filter_history_by_tag", { fileId, tagId }),

  listModels: () => invoke<ModelConfig[]>("list_models"),
  saveModelConfig: (config: ModelConfigInput) =>
    invoke<ModelConfig>("save_model_config", { config }),
  deleteModelConfig: (id: number) =>
    invoke<void>("delete_model_config", { id }),
  listOllamaModels: (baseUrl?: string) =>
    invoke<string[]>("list_ollama_models", { baseUrl }),

  runCompareTask: (payload: {
    projectId: number;
    promptContent: string;
    systemPrompt?: string;
    promptVersionHash?: string;
    modelIds: number[];
    params?: InferenceParams;
  }) =>
    invoke<CompareTask>("run_compare_task", {
      projectId: payload.projectId,
      promptContent: payload.promptContent,
      systemPrompt: payload.systemPrompt,
      promptVersionHash: payload.promptVersionHash,
      modelIds: payload.modelIds,
      params: payload.params,
    }),
  getCompareResult: (taskId: number) =>
    invoke<CompareTask>("get_compare_result", { taskId }),
  listCompareTasks: (projectId: number) =>
    invoke<CompareTask[]>("list_compare_tasks", { projectId }),
  scoreCompareResult: (resultId: number, scores: ScoreInput) =>
    invoke<CompareResult>("score_compare_result", { resultId, scores }),

  extractTemplateVariables: (template: string) =>
    invoke<string[]>("extract_template_variables", { template }),
  generateVariableCases: (variableValues: Record<string, unknown[]>) =>
    invoke<VariableCase[]>("generate_variable_cases", { variableValues }),
  parseCsvCases: (csv: string) =>
    invoke<{ headers: string[]; cases: Record<string, string>[] }>(
      "parse_csv_cases",
      { csv },
    ),
  runBatchTest: (payload: {
    projectId: number;
    template: string;
    systemPrompt?: string;
    modelId: number;
    cases: VariableCase[];
    params?: InferenceParams;
    concurrency?: number;
  }) =>
    invoke<BatchTask>("run_batch_test", {
      projectId: payload.projectId,
      template: payload.template,
      systemPrompt: payload.systemPrompt,
      modelId: payload.modelId,
      cases: payload.cases,
      params: payload.params,
      concurrency: payload.concurrency,
    }),
  getBatchResult: (taskId: number) =>
    invoke<BatchTask>("get_batch_result", { taskId }),
  scoreBatchResult: (resultId: number, score: number) =>
    invoke<void>("score_batch_result", { resultId, score }),
  saveTestSuite: (
    projectId: number,
    name: string,
    variablesSchema: unknown,
    cases: unknown,
  ) =>
    invoke<number>("save_test_suite", {
      projectId,
      name,
      variablesSchema,
      cases,
    }),
  listTestSuites: (projectId: number) =>
    invoke<unknown[]>("list_test_suites", { projectId }),

  exportCodeSnippet: (request: ExportCodeRequest) =>
    invoke<string>("export_code_snippet", { request }),
  exportJson: (
    fileId: number,
    versionHash: string,
    modelName?: string,
    params?: InferenceParams,
  ) =>
    invoke<string>("export_json", { fileId, versionHash, modelName, params }),
  exportYaml: (fileId: number, versionHash: string) =>
    invoke<string>("export_yaml", { fileId, versionHash }),
  exportMarkdownReport: (
    promptName: string,
    systemPrompt: string,
    userPrompt: string,
    params: InferenceParams,
    results: [string, string, number | null, number | null, string | null][],
  ) =>
    invoke<string>("export_markdown_report", {
      promptName,
      systemPrompt,
      userPrompt,
      params,
      results,
    }),
  exportPlainPrompt: (systemPrompt: string, userPrompt: string) =>
    invoke<string>("export_plain_prompt", { systemPrompt, userPrompt }),

  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settingsInput: AppSettings) =>
    invoke<void>("save_settings", { settingsInput }),
  setAppPassword: (password: string) =>
    invoke<void>("set_app_password", { password }),
  clearAppPassword: () => invoke<void>("clear_app_password"),
  verifyAppPassword: (password: string) =>
    invoke<boolean>("verify_app_password", { password }),
  backupDatabase: () => invoke<string>("backup_database"),
  listBackups: () => invoke<string[]>("list_backups"),
};
