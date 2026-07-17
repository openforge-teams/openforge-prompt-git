export interface Project {
  id: number;
  name: string;
  description?: string | null;
  created_at: string;
  updated_at: string;
}

export interface Folder {
  id: number;
  project_id: number;
  parent_id?: number | null;
  name: string;
  created_at: string;
}

export interface PromptFile {
  id: number;
  project_id: number;
  folder_id?: number | null;
  name: string;
  current_version_hash: string;
  system_prompt: string;
  user_prompt: string;
  created_at: string;
  updated_at: string;
}

export interface VersionTag {
  id: number;
  name: string;
  color: string;
  project_id: number;
}

export interface PromptVersion {
  id: number;
  version_hash: string;
  prompt_file_id: number;
  system_prompt: string;
  user_prompt: string;
  commit_message: string;
  remark?: string | null;
  created_at: string;
  tags: VersionTag[];
}

export interface DiffLine {
  kind: "equal" | "insert" | "delete" | string;
  old_line?: string | null;
  new_line?: string | null;
  old_no?: number | null;
  new_no?: number | null;
}

export interface VersionDiff {
  system_diff: DiffLine[];
  user_diff: DiffLine[];
  version_a: PromptVersion;
  version_b: PromptVersion;
}

export interface InferenceParams {
  temperature: number;
  max_tokens: number;
  top_p: number;
  frequency_penalty: number;
}

export interface ModelConfig {
  id: number;
  model_type: string;
  model_name: string;
  api_base?: string | null;
  api_key_masked: string;
  has_api_key: boolean;
  default_params: InferenceParams;
  is_enabled: boolean;
  created_at: string;
}

export interface ModelConfigInput {
  id?: number;
  model_type: string;
  model_name: string;
  api_base?: string | null;
  api_key?: string | null;
  default_params?: InferenceParams;
  is_enabled?: boolean;
}

export interface CompareResult {
  id: number;
  task_id: number;
  model_config_id: number;
  model_name: string;
  output_content?: string | null;
  scores?: Record<string, number> | null;
  total_score?: number | null;
  evaluation?: string | null;
  latency?: number | null;
  status: string;
  error_msg?: string | null;
  is_best: boolean;
  created_at: string;
}

export interface CompareTask {
  id: number;
  project_id: number;
  prompt_version_hash?: string | null;
  prompt_content: string;
  system_prompt?: string | null;
  models: number[];
  params: InferenceParams;
  status: string;
  created_at: string;
  results: CompareResult[];
}

export interface VariableCase {
  variables: Record<string, string | number | boolean | null>;
}

export interface BatchResult {
  id: number;
  task_id: number;
  case_index: number;
  variables: Record<string, unknown>;
  rendered_prompt: string;
  output_content?: string | null;
  score?: number | null;
  latency?: number | null;
  status: string;
  error_msg?: string | null;
}

export interface BatchTask {
  id: number;
  project_id: number;
  template: string;
  system_prompt?: string | null;
  model_config_id: number;
  params: InferenceParams;
  status: string;
  concurrency: number;
  created_at: string;
  results: BatchResult[];
}

export interface AppSettings {
  theme: string;
  auto_backup: boolean;
  backup_interval_hours: number;
  default_concurrency: number;
  ollama_base: string;
  has_app_password: boolean;
}

export interface ScoreInput {
  accuracy?: number;
  instruction?: number;
  format?: number;
  speed?: number;
  evaluation?: string;
  is_best?: boolean;
}

export interface ExportCodeRequest {
  system_prompt: string;
  user_prompt: string;
  model_name: string;
  language: string;
  params: InferenceParams;
}

export type AppView = "editor" | "compare" | "batch" | "settings";

export const DEFAULT_PARAMS: InferenceParams = {
  temperature: 0.7,
  max_tokens: 2048,
  top_p: 1,
  frequency_penalty: 0,
};
