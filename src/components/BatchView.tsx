import { useEffect, useMemo, useState } from "react";
import { Play, Save } from "lucide-react";
import { api } from "../lib/api";
import { useAppStore } from "../store";
import type { BatchTask, InferenceParams, VariableCase } from "../types";
import { DEFAULT_PARAMS } from "../types";
import { Button, Field, inputClass, textareaClass } from "./ui";

export function BatchView() {
  const {
    currentProjectId,
    systemPrompt,
    userPrompt,
    models,
    setError,
    setView,
    setPrompts,
    currentFileId,
    refreshVersions,
  } = useAppStore();

  const [template, setTemplate] = useState(userPrompt);
  const [variables, setVariables] = useState<string[]>([]);
  const [valueMap, setValueMap] = useState<Record<string, string>>({});
  const [modelId, setModelId] = useState<number | "">("");
  const [params, setParams] = useState<InferenceParams>(DEFAULT_PARAMS);
  const [concurrency, setConcurrency] = useState(3);
  const [task, setTask] = useState<BatchTask | null>(null);
  const [running, setRunning] = useState(false);
  const [csv, setCsv] = useState("");
  const [manualCases, setManualCases] = useState<VariableCase[] | null>(null);

  useEffect(() => {
    setTemplate(userPrompt);
  }, [userPrompt]);

  useEffect(() => {
    api.extractTemplateVariables(template).then((vars) => {
      setVariables(vars);
      setValueMap((prev) => {
        const next = { ...prev };
        for (const v of vars) {
          if (!(v in next)) next[v] = "";
        }
        return next;
      });
    });
  }, [template]);

  useEffect(() => {
    if (!modelId && models[0]) setModelId(models[0].id);
  }, [models, modelId]);

  const casesPreview = useMemo(() => {
    if (manualCases) return manualCases;
    const obj: Record<string, string[]> = {};
    for (const v of variables) {
      obj[v] = (valueMap[v] || "")
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean);
    }
    return null;
  }, [manualCases, variables, valueMap]);

  async function buildCases(): Promise<VariableCase[]> {
    if (manualCases) return manualCases;
    const variableValues: Record<string, string[]> = {};
    for (const v of variables) {
      variableValues[v] = (valueMap[v] || "")
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean);
      if (!variableValues[v].length) variableValues[v] = [""];
    }
    return api.generateVariableCases(variableValues);
  }

  async function importCsv() {
    try {
      const parsed = await api.parseCsvCases(csv);
      setManualCases(parsed.cases.map((c) => ({ variables: c })));
    } catch (e) {
      setError(String(e));
    }
  }

  async function run() {
    if (!currentProjectId || !modelId) return;
    setRunning(true);
    try {
      const cases = await buildCases();
      const result = await api.runBatchTest({
        projectId: currentProjectId,
        template,
        systemPrompt: systemPrompt || undefined,
        modelId: Number(modelId),
        cases,
        params,
        concurrency,
      });
      setTask(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }

  async function saveBestAsVersion(resultId: number) {
    if (!currentFileId || !task) return;
    const row = task.results.find((r) => r.id === resultId);
    if (!row) return;
    try {
      await api.commitPromptVersion({
        fileId: currentFileId,
        systemPrompt,
        userPrompt: row.rendered_prompt,
        commitMessage: `Best batch case #${row.case_index}`,
        remark: "Saved from batch test",
      });
      await refreshVersions();
      setPrompts(systemPrompt, row.rendered_prompt, false);
      alert("Saved as new version");
    } catch (e) {
      setError(String(e));
    }
  }

  async function persistSuite() {
    if (!currentProjectId) return;
    const name = prompt("Test suite name?");
    if (!name) return;
    try {
      const cases = await buildCases();
      await api.saveTestSuite(currentProjectId, name, { variables }, cases);
      alert("Test suite saved");
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex items-center justify-between border-b border-[var(--border)] px-4 py-3">
        <div>
          <h2 className="text-lg font-semibold">Batch Variable Test</h2>
          <p className="text-sm text-[var(--text-muted)]">
            Cartesian product / CSV cases against one model.
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" onClick={() => setView("editor")}>
            Back to Editor
          </Button>
          <Button variant="secondary" onClick={persistSuite}>
            <Save size={14} /> Save Suite
          </Button>
          <Button disabled={running || !modelId} onClick={run}>
            <Play size={14} /> {running ? "Running..." : "Run Batch"}
          </Button>
        </div>
      </header>

      <div className="grid flex-1 grid-cols-[1fr_1.2fr] overflow-hidden">
        <aside className="overflow-auto border-r border-[var(--border)] p-4">
          <Field label="Template">
            <textarea
              className={`${textareaClass} min-h-[160px]`}
              value={template}
              onChange={(e) => setTemplate(e.target.value)}
            />
          </Field>

          <Field label="Model">
            <select
              className={inputClass}
              value={modelId}
              onChange={(e) => setModelId(Number(e.target.value))}
            >
              {models.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.model_name} ({m.model_type})
                </option>
              ))}
            </select>
          </Field>

          <div className="mb-3 grid grid-cols-2 gap-2">
            <Field label="Concurrency">
              <input
                type="number"
                className={inputClass}
                min={1}
                max={10}
                value={concurrency}
                onChange={(e) => setConcurrency(Number(e.target.value))}
              />
            </Field>
            <Field label="Temperature">
              <input
                type="number"
                className={inputClass}
                step={0.1}
                value={params.temperature}
                onChange={(e) =>
                  setParams({ ...params, temperature: Number(e.target.value) })
                }
              />
            </Field>
          </div>

          <div className="mb-2 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
            Variable Values (one per line)
          </div>
          {variables.map((v) => (
            <Field key={v} label={`{{${v}}}`}>
              <textarea
                className={`${textareaClass} min-h-[70px]`}
                value={valueMap[v] || ""}
                onChange={(e) =>
                  setValueMap((m) => ({ ...m, [v]: e.target.value }))
                }
                placeholder="value1&#10;value2"
              />
            </Field>
          ))}

          <Field label="Or paste CSV (header row required)">
            <textarea
              className={`${textareaClass} min-h-[100px]`}
              value={csv}
              onChange={(e) => setCsv(e.target.value)}
              placeholder={"name,city\nAda,Paris"}
            />
          </Field>
          <Button variant="secondary" onClick={importCsv}>
            Import CSV Cases
          </Button>
          {manualCases ? (
            <button
              className="ml-2 text-xs text-[var(--accent)]"
              onClick={() => setManualCases(null)}
            >
              Clear CSV cases ({manualCases.length})
            </button>
          ) : null}
          {casesPreview ? (
            <div className="mt-2 text-xs text-[var(--text-muted)]">
              Using {casesPreview.length} imported cases
            </div>
          ) : null}
        </aside>

        <main className="overflow-auto p-4">
          {!task ? (
            <div className="py-20 text-center text-[var(--text-muted)]">
              Configure variables and run a batch test.
            </div>
          ) : (
            <div className="overflow-auto rounded-xl border border-[var(--border)]">
              <table className="min-w-full text-left text-sm">
                <thead className="bg-[var(--bg-panel)] text-xs uppercase text-[var(--text-muted)]">
                  <tr>
                    <th className="px-3 py-2">#</th>
                    <th className="px-3 py-2">Variables</th>
                    <th className="px-3 py-2">Output</th>
                    <th className="px-3 py-2">Latency</th>
                    <th className="px-3 py-2">Score</th>
                    <th className="px-3 py-2">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {[...task.results]
                    .sort((a, b) => (b.score ?? -1) - (a.score ?? -1))
                    .map((r) => (
                      <tr key={r.id} className="border-t border-[var(--border)] align-top">
                        <td className="px-3 py-2 mono">{r.case_index}</td>
                        <td className="px-3 py-2 mono text-xs">
                          {JSON.stringify(r.variables)}
                        </td>
                        <td className="max-w-md px-3 py-2 text-xs">
                          {r.status === "failed"
                            ? r.error_msg
                            : (r.output_content || "").slice(0, 280)}
                        </td>
                        <td className="px-3 py-2">{r.latency ?? "-"}</td>
                        <td className="px-3 py-2">
                          <input
                            type="number"
                            className={`${inputClass} w-20`}
                            defaultValue={r.score ?? ""}
                            onBlur={async (e) => {
                              const score = Number(e.target.value);
                              if (Number.isNaN(score)) return;
                              try {
                                await api.scoreBatchResult(r.id, score);
                                setTask(await api.getBatchResult(task.id));
                              } catch (err) {
                                setError(String(err));
                              }
                            }}
                          />
                        </td>
                        <td className="px-3 py-2">
                          <Button
                            variant="ghost"
                            className="!px-2 !py-1 text-xs"
                            onClick={() => saveBestAsVersion(r.id)}
                          >
                            Save version
                          </Button>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
