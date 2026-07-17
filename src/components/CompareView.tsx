import { useEffect, useState } from "react";
import { Play, Star } from "lucide-react";
import { api } from "../lib/api";
import { useAppStore } from "../store";
import type { CompareTask, InferenceParams } from "../types";
import { DEFAULT_PARAMS } from "../types";
import { Button, Field, inputClass, textareaClass } from "./ui";

export function CompareView() {
  const {
    currentProjectId,
    systemPrompt,
    userPrompt,
    models,
    versions,
    setError,
    setView,
  } = useAppStore();

  const [selected, setSelected] = useState<number[]>([]);
  const [params, setParams] = useState<InferenceParams>(DEFAULT_PARAMS);
  const [task, setTask] = useState<CompareTask | null>(null);
  const [history, setHistory] = useState<CompareTask[]>([]);
  const [running, setRunning] = useState(false);
  const [layout, setLayout] = useState<"grid" | "columns">("grid");

  useEffect(() => {
    if (!currentProjectId) return;
    api
      .listCompareTasks(currentProjectId)
      .then(setHistory)
      .catch((e) => setError(String(e)));
  }, [currentProjectId, task, setError]);

  async function run() {
    if (!currentProjectId || selected.length === 0) return;
    setRunning(true);
    try {
      const result = await api.runCompareTask({
        projectId: currentProjectId,
        promptContent: userPrompt,
        systemPrompt: systemPrompt || undefined,
        promptVersionHash: versions[0]?.version_hash,
        modelIds: selected,
        params,
      });
      setTask(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }

  async function score(
    resultId: number,
    field: "accuracy" | "instruction" | "format" | "speed",
    value: number,
  ) {
    try {
      await api.scoreCompareResult(resultId, { [field]: value });
      if (task) {
        const refreshed = await api.getCompareResult(task.id);
        setTask(refreshed);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function markBest(resultId: number) {
    try {
      await api.scoreCompareResult(resultId, { is_best: true });
      if (task) setTask(await api.getCompareResult(task.id));
    } catch (e) {
      setError(String(e));
    }
  }

  async function exportReport() {
    if (!task) return;
    const md = await api.exportMarkdownReport(
      "Compare Run",
      systemPrompt,
      userPrompt,
      params,
      task.results.map((r) => [
        r.model_name,
        r.output_content || r.error_msg || "",
        r.total_score ?? null,
        r.latency ?? null,
        r.evaluation ?? null,
      ]),
    );
    await navigator.clipboard.writeText(md);
    alert("Markdown report copied to clipboard");
  }

  const enabledModels = models.filter((m) => m.is_enabled);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="border-b border-[var(--border)] px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold">Multi-Model Compare</h2>
            <p className="text-sm text-[var(--text-muted)]">
              Run the current prompt against multiple models concurrently.
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="secondary" onClick={() => setView("editor")}>
              Back to Editor
            </Button>
            <Button disabled={running || selected.length === 0} onClick={run}>
              <Play size={14} /> {running ? "Running..." : "Run Compare"}
            </Button>
          </div>
        </div>
      </header>

      <div className="grid flex-1 grid-cols-[320px_1fr] overflow-hidden">
        <aside className="overflow-auto border-r border-[var(--border)] p-4">
          <div className="mb-3 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
            Models
          </div>
          <div className="mb-4 space-y-2">
            {enabledModels.map((m) => {
              const on = selected.includes(m.id);
              return (
                <label
                  key={m.id}
                  className={`flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-sm ${
                    on
                      ? "border-[var(--accent)] bg-[var(--accent-soft)]"
                      : "border-[var(--border)]"
                  }`}
                >
                  <input
                    type="checkbox"
                    checked={on}
                    onChange={() =>
                      setSelected((ids) =>
                        on ? ids.filter((x) => x !== m.id) : [...ids, m.id],
                      )
                    }
                  />
                  <span className="truncate">
                    {m.model_name}
                    <span className="ml-1 text-xs text-[var(--text-muted)]">
                      ({m.model_type})
                    </span>
                  </span>
                </label>
              );
            })}
            {!enabledModels.length ? (
              <div className="text-sm text-[var(--text-muted)]">
                Configure models in Settings first.
              </div>
            ) : null}
          </div>

          <div className="mb-3 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
            Inference Params
          </div>
          {(
            [
              ["temperature", 0, 2, 0.1],
              ["max_tokens", 64, 8192, 64],
              ["top_p", 0, 1, 0.05],
              ["frequency_penalty", 0, 2, 0.1],
            ] as const
          ).map(([key, min, max, step]) => (
            <Field key={key} label={key}>
              <input
                type="number"
                className={inputClass}
                min={min}
                max={max}
                step={step}
                value={params[key]}
                onChange={(e) =>
                  setParams({ ...params, [key]: Number(e.target.value) })
                }
              />
            </Field>
          ))}

          <div className="mt-4">
            <div className="mb-2 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
              History
            </div>
            <div className="space-y-2">
              {history.slice(0, 8).map((h) => (
                <button
                  key={h.id}
                  className="block w-full rounded-md border border-[var(--border)] px-2 py-2 text-left text-xs hover:bg-[var(--bg-hover)]"
                  onClick={() => setTask(h)}
                >
                  <div>#{h.id} · {h.status}</div>
                  <div className="text-[var(--text-muted)]">{h.created_at}</div>
                </button>
              ))}
            </div>
          </div>
        </aside>

        <main className="overflow-auto p-4">
          <div className="mb-3 flex items-center gap-2">
            <Button
              variant={layout === "grid" ? "primary" : "secondary"}
              onClick={() => setLayout("grid")}
            >
              Grid
            </Button>
            <Button
              variant={layout === "columns" ? "primary" : "secondary"}
              onClick={() => setLayout("columns")}
            >
              Columns
            </Button>
            {task ? (
              <Button variant="secondary" onClick={exportReport}>
                Copy Markdown Report
              </Button>
            ) : null}
          </div>

          <div className="mb-4 rounded-lg border border-[var(--border)] bg-[var(--editor)] p-3">
            <div className="mb-1 text-xs text-[var(--text-muted)]">Current Prompt</div>
            <pre className="mono max-h-28 overflow-auto whitespace-pre-wrap text-xs">
              {userPrompt || "Empty prompt — edit it in the Editor first."}
            </pre>
          </div>

          {!task ? (
            <div className="py-16 text-center text-[var(--text-muted)]">
              Select models and run a compare task.
            </div>
          ) : (
            <div
              className={
                layout === "grid"
                  ? "grid grid-cols-1 gap-3 xl:grid-cols-2"
                  : "flex gap-3 overflow-x-auto"
              }
            >
              {task.results.map((r) => (
                <div
                  key={r.id}
                  className={`rounded-xl border bg-[var(--bg-elevated)] p-4 ${
                    r.is_best ? "border-[var(--warning)]" : "border-[var(--border)]"
                  } ${layout === "columns" ? "min-w-[360px] flex-1" : ""}`}
                >
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <div>
                      <div className="font-medium">{r.model_name}</div>
                      <div className="text-xs text-[var(--text-muted)]">
                        {r.status}
                        {r.latency != null ? ` · ${r.latency}ms` : ""}
                        {r.total_score != null
                          ? ` · score ${r.total_score.toFixed(1)}`
                          : ""}
                      </div>
                    </div>
                    <Button variant="ghost" onClick={() => markBest(r.id)}>
                      <Star
                        size={14}
                        className={r.is_best ? "fill-[var(--warning)] text-[var(--warning)]" : ""}
                      />
                    </Button>
                  </div>
                  {r.status === "failed" ? (
                    <div className="text-sm text-[var(--danger)]">{r.error_msg}</div>
                  ) : (
                    <textarea
                      className={`${textareaClass} min-h-[160px]`}
                      readOnly
                      value={r.output_content || ""}
                    />
                  )}
                  <div className="mt-3 grid grid-cols-2 gap-2">
                    {(
                      [
                        ["accuracy", "Accuracy"],
                        ["instruction", "Instruction"],
                        ["format", "Format"],
                        ["speed", "Speed"],
                      ] as const
                    ).map(([key, label]) => (
                      <label key={key} className="text-xs text-[var(--text-muted)]">
                        {label}
                        <input
                          type="number"
                          min={0}
                          max={10}
                          step={0.5}
                          className={`${inputClass} mt-1`}
                          defaultValue={r.scores?.[key] ?? ""}
                          onBlur={(e) => {
                            const v = Number(e.target.value);
                            if (!Number.isNaN(v)) score(r.id, key, v);
                          }}
                        />
                      </label>
                    ))}
                  </div>
                  <textarea
                    className={`${textareaClass} mt-2 min-h-[60px]`}
                    placeholder="Evaluation notes..."
                    defaultValue={r.evaluation || ""}
                    onBlur={async (e) => {
                      try {
                        await api.scoreCompareResult(r.id, {
                          evaluation: e.target.value,
                        });
                      } catch (err) {
                        setError(String(err));
                      }
                    }}
                  />
                </div>
              ))}
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
