import { useEffect, useMemo, useState } from "react";
import {
  Copy,
  Diff,
  Download,
  GitCommitHorizontal,
  History,
  RotateCcw,
  Save,
} from "lucide-react";
import { api } from "../lib/api";
import { useAppStore } from "../store";
import type { DiffLine, PromptVersion, VersionTag } from "../types";
import { Button, Field, Modal, inputClass, textareaClass } from "./ui";

function DiffBlock({ title, lines }: { title: string; lines: DiffLine[] }) {
  return (
    <div className="mb-4 overflow-hidden rounded-lg border border-[var(--border)]">
      <div className="border-b border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
        {title}
      </div>
      <div className="max-h-64 overflow-auto mono text-xs">
        {lines.length === 0 ? (
          <div className="px-3 py-4 text-[var(--text-muted)]">No changes</div>
        ) : (
          lines.map((line, i) => (
            <div
              key={i}
              className={`flex whitespace-pre-wrap px-2 py-0.5 ${
                line.kind === "insert"
                  ? "bg-emerald-500/15 text-emerald-300"
                  : line.kind === "delete"
                    ? "bg-rose-500/15 text-rose-300"
                    : "text-[var(--text-muted)]"
              }`}
            >
              <span className="w-10 shrink-0 opacity-50">{line.old_no ?? ""}</span>
              <span className="w-10 shrink-0 opacity-50">{line.new_no ?? ""}</span>
              <span className="w-4 shrink-0">
                {line.kind === "insert" ? "+" : line.kind === "delete" ? "-" : " "}
              </span>
              <span>{line.new_line ?? line.old_line ?? ""}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export function EditorView() {
  const {
    currentFileId,
    currentProjectId,
    files,
    systemPrompt,
    userPrompt,
    dirty,
    versions,
    setPrompts,
    refreshVersions,
    selectFile,
    setError,
    rightTab,
    setRightTab,
  } = useAppStore();

  const [commitOpen, setCommitOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [remark, setRemark] = useState("");
  const [tags, setTags] = useState<VersionTag[]>([]);
  const [selectedTagIds, setSelectedTagIds] = useState<number[]>([]);
  const [filterTagId, setFilterTagId] = useState<number | null>(null);
  const [diffOpen, setDiffOpen] = useState(false);
  const [diffA, setDiffA] = useState("");
  const [diffB, setDiffB] = useState("");
  const [diffLines, setDiffLines] = useState<{
    system_diff: DiffLine[];
    user_diff: DiffLine[];
  } | null>(null);
  const [variables, setVariables] = useState<string[]>([]);
  const [exportLang, setExportLang] = useState("python");
  const [exportContent, setExportContent] = useState("");
  const [busy, setBusy] = useState(false);

  const currentFile = files.find((f) => f.id === currentFileId);

  useEffect(() => {
    if (!currentProjectId) return;
    api.listTags(currentProjectId).then(setTags).catch((e) => setError(String(e)));
  }, [currentProjectId, setError]);

  useEffect(() => {
    api
      .extractTemplateVariables(userPrompt)
      .then(setVariables)
      .catch(() => setVariables([]));
  }, [userPrompt]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        if (currentFileId) setCommitOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [currentFileId]);

  const shownVersions = useMemo(() => {
    if (!filterTagId) return versions;
    return versions.filter((v) => v.tags.some((t) => t.id === filterTagId));
  }, [versions, filterTagId]);

  async function commit() {
    if (!currentFileId) return;
    setBusy(true);
    try {
      await api.commitPromptVersion({
        fileId: currentFileId,
        systemPrompt,
        userPrompt,
        commitMessage: message,
        remark: remark || undefined,
        tagIds: selectedTagIds,
      });
      setCommitOpen(false);
      setMessage("");
      setRemark("");
      setSelectedTagIds([]);
      await refreshVersions();
      await selectFile(currentFileId);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function rollback(v: PromptVersion) {
    if (!currentFileId) return;
    if (!confirm(`Rollback to ${v.version_hash.slice(0, 8)}? Current content will be snapshotted.`))
      return;
    try {
      await api.rollbackVersion(currentFileId, v.version_hash);
      await selectFile(currentFileId);
      await refreshVersions();
    } catch (e) {
      setError(String(e));
    }
  }

  async function showDiff(a: string, b: string) {
    try {
      const diff = await api.diffVersions(a, b);
      setDiffA(a);
      setDiffB(b);
      setDiffLines({ system_diff: diff.system_diff, user_diff: diff.user_diff });
      setDiffOpen(true);
    } catch (e) {
      setError(String(e));
    }
  }

  async function doExport(kind: string) {
    if (!currentFileId || !versions[0]) return;
    try {
      if (kind === "code") {
        const code = await api.exportCodeSnippet({
          system_prompt: systemPrompt,
          user_prompt: userPrompt,
          model_name: "gpt-4o-mini",
          language: exportLang,
          params: {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 1,
            frequency_penalty: 0,
          },
        });
        setExportContent(code);
      } else if (kind === "json") {
        setExportContent(
          await api.exportJson(currentFileId, versions[0].version_hash),
        );
      } else if (kind === "yaml") {
        setExportContent(
          await api.exportYaml(currentFileId, versions[0].version_hash),
        );
      } else {
        setExportContent(await api.exportPlainPrompt(systemPrompt, userPrompt));
      }
    } catch (e) {
      setError(String(e));
    }
  }

  if (!currentFileId) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--text-muted)]">
        Select or create a prompt file to begin.
      </div>
    );
  }

  return (
    <div className="flex h-full min-w-0 flex-1">
      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex items-center gap-2 border-b border-[var(--border)] px-4 py-2">
          <div className="min-w-0 flex-1">
            <div className="truncate font-medium">{currentFile?.name}</div>
            <div className="mono text-xs text-[var(--text-muted)]">
              {dirty ? "Uncommitted changes" : `HEAD ${currentFile?.current_version_hash.slice(0, 8)}`}
            </div>
          </div>
          <Button variant="secondary" onClick={() => setExportOpen(true)}>
            <Download size={14} /> Export
          </Button>
          <Button onClick={() => setCommitOpen(true)}>
            <Save size={14} /> Commit
          </Button>
        </header>

        <div className="grid flex-1 grid-rows-2 gap-0 overflow-hidden">
          <div className="flex min-h-0 flex-col border-b border-[var(--border)]">
            <div className="px-4 py-2 text-xs uppercase tracking-wide text-[var(--text-muted)]">
              System Prompt
            </div>
            <textarea
              className="min-h-0 flex-1 resize-none border-0 bg-[var(--editor)] px-4 pb-3 mono text-sm outline-none"
              value={systemPrompt}
              onChange={(e) => setPrompts(e.target.value, userPrompt)}
              placeholder="Optional system instructions..."
            />
          </div>
          <div className="flex min-h-0 flex-col">
            <div className="px-4 py-2 text-xs uppercase tracking-wide text-[var(--text-muted)]">
              User Prompt
            </div>
            <textarea
              className="min-h-0 flex-1 resize-none border-0 bg-[var(--editor)] px-4 pb-3 mono text-sm outline-none"
              value={userPrompt}
              onChange={(e) => setPrompts(systemPrompt, e.target.value)}
              placeholder="Write your prompt. Use {{variable}} for templates."
            />
          </div>
        </div>
      </section>

      <aside className="flex w-80 flex-col border-l border-[var(--border)] bg-[var(--bg-panel)]/70">
        <div className="flex border-b border-[var(--border)]">
          {(
            [
              ["history", History, "History"],
              ["variables", Diff, "Vars"],
              ["models", GitCommitHorizontal, "Tips"],
            ] as const
          ).map(([id, Icon, label]) => (
            <button
              key={id}
              className={`flex flex-1 items-center justify-center gap-1 py-2 text-xs ${
                rightTab === id
                  ? "border-b-2 border-[var(--accent)] text-[var(--accent)]"
                  : "text-[var(--text-muted)]"
              }`}
              onClick={() => setRightTab(id)}
            >
              <Icon size={13} /> {label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-auto p-3">
          {rightTab === "history" && (
            <>
              <div className="mb-2">
                <select
                  className={inputClass}
                  value={filterTagId ?? ""}
                  onChange={(e) =>
                    setFilterTagId(e.target.value ? Number(e.target.value) : null)
                  }
                >
                  <option value="">All tags</option>
                  {tags.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                {shownVersions.map((v, idx) => (
                  <div
                    key={v.version_hash}
                    className="rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] p-3"
                  >
                    <div className="mono text-xs text-[var(--accent)]">
                      {v.version_hash.slice(0, 8)}
                    </div>
                    <div className="mt-1 text-sm font-medium">{v.commit_message}</div>
                    <div className="mt-1 text-xs text-[var(--text-muted)]">{v.created_at}</div>
                    {v.remark ? (
                      <div className="mt-2 text-xs text-[var(--warning)]">{v.remark}</div>
                    ) : null}
                    <div className="mt-2 flex flex-wrap gap-1">
                      {v.tags.map((t) => (
                        <span
                          key={t.id}
                          className="rounded px-1.5 py-0.5 text-[10px]"
                          style={{ background: `${t.color}33`, color: t.color }}
                        >
                          {t.name}
                        </span>
                      ))}
                    </div>
                    <div className="mt-3 flex flex-wrap gap-1">
                      <Button
                        variant="ghost"
                        className="!px-2 !py-1 text-xs"
                        onClick={() => setPrompts(v.system_prompt, v.user_prompt, true)}
                      >
                        Load
                      </Button>
                      <Button
                        variant="ghost"
                        className="!px-2 !py-1 text-xs"
                        onClick={() => rollback(v)}
                      >
                        <RotateCcw size={12} /> Rollback
                      </Button>
                      {idx < shownVersions.length - 1 ? (
                        <Button
                          variant="ghost"
                          className="!px-2 !py-1 text-xs"
                          onClick={() =>
                            showDiff(
                              shownVersions[idx + 1].version_hash,
                              v.version_hash,
                            )
                          }
                        >
                          Diff prev
                        </Button>
                      ) : null}
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}

          {rightTab === "variables" && (
            <div>
              <div className="mb-2 text-xs text-[var(--text-muted)]">
                Detected template variables
              </div>
              {variables.length === 0 ? (
                <div className="text-sm text-[var(--text-muted)]">
                  No {"{{variables}}"} found.
                </div>
              ) : (
                <ul className="space-y-1">
                  {variables.map((v) => (
                    <li
                      key={v}
                      className="rounded-md bg-[var(--accent-soft)] px-2 py-1 mono text-sm text-[var(--accent)]"
                    >
                      {`{{${v}}}`}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}

          {rightTab === "models" && (
            <div className="space-y-3 text-sm text-[var(--text-muted)]">
              <p>
                Use <kbd className="rounded bg-[var(--bg-hover)] px-1">⌘/Ctrl+S</kbd> to
                commit a version.
              </p>
              <p>Open Compare to run the current prompt across multiple models.</p>
              <p>Open Batch Test for {"{{variable}}"} matrix runs.</p>
            </div>
          )}
        </div>
      </aside>

      <Modal
        open={commitOpen}
        title="Commit Version"
        onClose={() => setCommitOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setCommitOpen(false)}>
              Cancel
            </Button>
            <Button disabled={!message.trim() || busy} onClick={commit}>
              <GitCommitHorizontal size={14} /> Commit
            </Button>
          </>
        }
      >
        <Field label="Commit message *">
          <input
            className={inputClass}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Improve instruction clarity"
          />
        </Field>
        <Field label="Remark">
          <textarea
            className={textareaClass}
            value={remark}
            onChange={(e) => setRemark(e.target.value)}
            placeholder="Optional test notes..."
          />
        </Field>
        <div className="text-sm text-[var(--text-muted)]">Tags</div>
        <div className="mt-2 flex flex-wrap gap-2">
          {tags.map((t) => {
            const on = selectedTagIds.includes(t.id);
            return (
              <button
                key={t.id}
                className={`rounded-md border px-2 py-1 text-xs ${
                  on ? "border-transparent text-white" : "border-[var(--border)]"
                }`}
                style={on ? { background: t.color } : undefined}
                onClick={() =>
                  setSelectedTagIds((ids) =>
                    on ? ids.filter((x) => x !== t.id) : [...ids, t.id],
                  )
                }
              >
                {t.name}
              </button>
            );
          })}
        </div>
      </Modal>

      <Modal open={diffOpen} title="Version Diff" onClose={() => setDiffOpen(false)}>
        <div className="mb-3 mono text-xs text-[var(--text-muted)]">
          {diffA.slice(0, 8)} → {diffB.slice(0, 8)}
        </div>
        {diffLines ? (
          <>
            <DiffBlock title="System Prompt" lines={diffLines.system_diff} />
            <DiffBlock title="User Prompt" lines={diffLines.user_diff} />
          </>
        ) : null}
      </Modal>

      <Modal
        open={exportOpen}
        title="Export"
        onClose={() => setExportOpen(false)}
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => navigator.clipboard.writeText(exportContent)}
              disabled={!exportContent}
            >
              <Copy size={14} /> Copy
            </Button>
            <Button variant="ghost" onClick={() => setExportOpen(false)}>
              Close
            </Button>
          </>
        }
      >
        <div className="mb-3 flex flex-wrap gap-2">
          <Button variant="secondary" onClick={() => doExport("code")}>
            Code
          </Button>
          <Button variant="secondary" onClick={() => doExport("json")}>
            JSON
          </Button>
          <Button variant="secondary" onClick={() => doExport("yaml")}>
            YAML
          </Button>
          <Button variant="secondary" onClick={() => doExport("plain")}>
            Plain
          </Button>
        </div>
        <Field label="Code language">
          <select
            className={inputClass}
            value={exportLang}
            onChange={(e) => setExportLang(e.target.value)}
          >
            {["python", "javascript", "typescript", "go", "java", "rust", "curl"].map(
              (l) => (
                <option key={l} value={l}>
                  {l}
                </option>
              ),
            )}
          </select>
        </Field>
        <textarea
          className={`${textareaClass} min-h-[220px]`}
          value={exportContent}
          readOnly
          placeholder="Choose an export format..."
        />
      </Modal>
    </div>
  );
}
