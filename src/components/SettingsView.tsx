import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { useAppStore } from "../store";
import type { AppSettings, ModelConfigInput } from "../types";
import { DEFAULT_PARAMS } from "../types";
import { Button, Field, inputClass } from "./ui";

const MODEL_TYPES = [
  "openai",
  "deepseek",
  "claude",
  "qwen",
  "doubao",
  "wenxin",
  "ollama",
  "custom",
];

export function SettingsView() {
  const { models, refreshModels, settings, applyTheme, setError, bootstrap } =
    useAppStore();
  const [form, setForm] = useState<ModelConfigInput>({
    model_type: "openai",
    model_name: "gpt-4o-mini",
    api_base: "",
    api_key: "",
    default_params: DEFAULT_PARAMS,
    is_enabled: true,
  });
  const [localSettings, setLocalSettings] = useState<AppSettings | null>(settings);
  const [password, setPassword] = useState("");
  const [ollamaModels, setOllamaModels] = useState<string[]>([]);
  const [backups, setBackups] = useState<string[]>([]);
  const [tagName, setTagName] = useState("");
  const [tagColor, setTagColor] = useState("#3d9cf0");
  const currentProjectId = useAppStore((s) => s.currentProjectId);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  useEffect(() => {
    api.listBackups().then(setBackups).catch(() => undefined);
  }, []);

  async function saveModel() {
    try {
      await api.saveModelConfig(form);
      setForm({
        model_type: "openai",
        model_name: "gpt-4o-mini",
        api_base: "",
        api_key: "",
        default_params: DEFAULT_PARAMS,
        is_enabled: true,
      });
      await refreshModels();
    } catch (e) {
      setError(String(e));
    }
  }

  async function removeModel(id: number) {
    if (!confirm("Delete this model config?")) return;
    try {
      await api.deleteModelConfig(id);
      await refreshModels();
    } catch (e) {
      setError(String(e));
    }
  }

  async function savePrefs() {
    if (!localSettings) return;
    try {
      await api.saveSettings(localSettings);
      applyTheme(localSettings.theme);
      await bootstrap();
    } catch (e) {
      setError(String(e));
    }
  }

  async function discoverOllama() {
    try {
      const base = localSettings?.ollama_base || "http://127.0.0.1:11434";
      const list = await api.listOllamaModels(base);
      setOllamaModels(list);
    } catch (e) {
      setError(String(e));
    }
  }

  async function addOllama(name: string) {
    try {
      await api.saveModelConfig({
        model_type: "ollama",
        model_name: name,
        api_base: localSettings?.ollama_base || "http://127.0.0.1:11434",
        is_enabled: true,
        default_params: DEFAULT_PARAMS,
      });
      await refreshModels();
    } catch (e) {
      setError(String(e));
    }
  }

  async function createTag() {
    if (!currentProjectId || !tagName.trim()) return;
    try {
      await api.createTag(currentProjectId, tagName.trim(), tagColor);
      setTagName("");
      alert("Tag created for current project");
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="h-full overflow-auto p-6">
      <h2 className="mb-1 text-xl font-semibold">Settings</h2>
      <p className="mb-6 text-sm text-[var(--text-muted)]">
        Models, privacy, backups, and preferences. All data stays on this machine.
      </p>

      <div className="grid gap-6 xl:grid-cols-2">
        <section className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
          <h3 className="mb-3 font-medium">Model Configurations</h3>
          <div className="mb-4 space-y-2">
            {models.map((m) => (
              <div
                key={m.id}
                className="flex items-center justify-between rounded-md border border-[var(--border)] px-3 py-2 text-sm"
              >
                <div>
                  <div className="font-medium">
                    {m.model_name}{" "}
                    <span className="text-xs text-[var(--text-muted)]">
                      ({m.model_type})
                    </span>
                  </div>
                  <div className="text-xs text-[var(--text-muted)]">
                    key: {m.has_api_key ? m.api_key_masked : "none"} ·{" "}
                    {m.is_enabled ? "enabled" : "disabled"}
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    onClick={() =>
                      setForm({
                        id: m.id,
                        model_type: m.model_type,
                        model_name: m.model_name,
                        api_base: m.api_base || "",
                        api_key: "",
                        default_params: m.default_params,
                        is_enabled: m.is_enabled,
                      })
                    }
                  >
                    Edit
                  </Button>
                  <Button variant="danger" onClick={() => removeModel(m.id)}>
                    Delete
                  </Button>
                </div>
              </div>
            ))}
          </div>

          <Field label="Type">
            <select
              className={inputClass}
              value={form.model_type}
              onChange={(e) => setForm({ ...form, model_type: e.target.value })}
            >
              {MODEL_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Model name">
            <input
              className={inputClass}
              value={form.model_name}
              onChange={(e) => setForm({ ...form, model_name: e.target.value })}
            />
          </Field>
          <Field label="API base (optional)">
            <input
              className={inputClass}
              value={form.api_base || ""}
              onChange={(e) => setForm({ ...form, api_base: e.target.value })}
            />
          </Field>
          <Field label="API key (encrypted locally)">
            <input
              className={inputClass}
              type="password"
              value={form.api_key || ""}
              onChange={(e) => setForm({ ...form, api_key: e.target.value })}
              placeholder={form.id ? "Leave blank to keep existing" : ""}
            />
          </Field>
          <label className="mb-3 flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={form.is_enabled ?? true}
              onChange={(e) => setForm({ ...form, is_enabled: e.target.checked })}
            />
            Enabled
          </label>
          <Button onClick={saveModel}>{form.id ? "Update Model" : "Add Model"}</Button>
        </section>

        <section className="space-y-6">
          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
            <h3 className="mb-3 font-medium">Ollama Discovery</h3>
            <Field label="Ollama base URL">
              <input
                className={inputClass}
                value={localSettings?.ollama_base || ""}
                onChange={(e) =>
                  localSettings &&
                  setLocalSettings({ ...localSettings, ollama_base: e.target.value })
                }
              />
            </Field>
            <Button variant="secondary" onClick={discoverOllama}>
              List Local Models
            </Button>
            <div className="mt-3 flex flex-wrap gap-2">
              {ollamaModels.map((name) => (
                <button
                  key={name}
                  className="rounded-md border border-[var(--border)] px-2 py-1 text-xs hover:bg-[var(--bg-hover)]"
                  onClick={() => addOllama(name)}
                >
                  + {name}
                </button>
              ))}
            </div>
          </div>

          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
            <h3 className="mb-3 font-medium">Preferences</h3>
            {localSettings ? (
              <>
                <Field label="Theme">
                  <select
                    className={inputClass}
                    value={localSettings.theme}
                    onChange={(e) =>
                      setLocalSettings({ ...localSettings, theme: e.target.value })
                    }
                  >
                    <option value="system">System</option>
                    <option value="dark">Dark</option>
                    <option value="light">Light</option>
                  </select>
                </Field>
                <Field label="Default concurrency">
                  <input
                    type="number"
                    className={inputClass}
                    value={localSettings.default_concurrency}
                    onChange={(e) =>
                      setLocalSettings({
                        ...localSettings,
                        default_concurrency: Number(e.target.value),
                      })
                    }
                  />
                </Field>
                <label className="mb-3 flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={localSettings.auto_backup}
                    onChange={(e) =>
                      setLocalSettings({
                        ...localSettings,
                        auto_backup: e.target.checked,
                      })
                    }
                  />
                  Auto backup
                </label>
                <Button onClick={savePrefs}>Save Preferences</Button>
              </>
            ) : null}
          </div>

          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
            <h3 className="mb-3 font-medium">App Password</h3>
            <Field label="Password">
              <input
                type="password"
                className={inputClass}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </Field>
            <div className="flex gap-2">
              <Button
                onClick={async () => {
                  try {
                    await api.setAppPassword(password);
                    setPassword("");
                    await bootstrap();
                  } catch (e) {
                    setError(String(e));
                  }
                }}
              >
                Set Password
              </Button>
              <Button
                variant="secondary"
                onClick={async () => {
                  try {
                    await api.clearAppPassword();
                    await bootstrap();
                  } catch (e) {
                    setError(String(e));
                  }
                }}
              >
                Clear
              </Button>
            </div>
          </div>

          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
            <h3 className="mb-3 font-medium">Project Tags</h3>
            <div className="mb-2 flex gap-2">
              <input
                className={inputClass}
                placeholder="Tag name"
                value={tagName}
                onChange={(e) => setTagName(e.target.value)}
              />
              <input
                type="color"
                value={tagColor}
                onChange={(e) => setTagColor(e.target.value)}
              />
            </div>
            <Button variant="secondary" onClick={createTag}>
              Add Tag to Current Project
            </Button>
          </div>

          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
            <h3 className="mb-3 font-medium">Backup</h3>
            <Button
              variant="secondary"
              onClick={async () => {
                try {
                  const path = await api.backupDatabase();
                  alert(`Backup created: ${path}`);
                  setBackups(await api.listBackups());
                } catch (e) {
                  setError(String(e));
                }
              }}
            >
              Backup Now
            </Button>
            <ul className="mt-3 space-y-1 text-xs text-[var(--text-muted)]">
              {backups.map((b) => (
                <li key={b} className="mono truncate">
                  {b}
                </li>
              ))}
            </ul>
          </div>
        </section>
      </div>
    </div>
  );
}
