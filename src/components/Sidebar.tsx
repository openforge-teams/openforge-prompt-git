import {
  FileText,
  FolderPlus,
  GitBranch,
  Layers,
  Plus,
  Settings,
  SplitSquareHorizontal,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { api } from "../lib/api";
import { useAppStore } from "../store";
import { Button, Field, Modal, inputClass } from "./ui";

export function Sidebar() {
  const {
    projects,
    currentProjectId,
    folders,
    files,
    currentFileId,
    view,
    setView,
    selectProject,
    selectFile,
    refreshProjects,
    setError,
  } = useAppStore();

  const [projectOpen, setProjectOpen] = useState(false);
  const [fileOpen, setFileOpen] = useState(false);
  const [folderOpen, setFolderOpen] = useState(false);
  const [name, setName] = useState("");
  const [desc, setDesc] = useState("");

  const nav = [
    { id: "editor" as const, label: "Editor", icon: GitBranch },
    { id: "compare" as const, label: "Compare", icon: SplitSquareHorizontal },
    { id: "batch" as const, label: "Batch Test", icon: Layers },
    { id: "settings" as const, label: "Settings", icon: Settings },
  ];

  async function createProject() {
    try {
      await api.createProject(name.trim() || "Untitled Project", desc || undefined);
      setProjectOpen(false);
      setName("");
      setDesc("");
      await refreshProjects();
    } catch (e) {
      setError(String(e));
    }
  }

  async function createFile() {
    if (!currentProjectId) return;
    try {
      const file = await api.createPromptFile(
        currentProjectId,
        name.trim() || "untitled.prompt",
      );
      setFileOpen(false);
      setName("");
      await refreshProjects();
      await selectFile(file.id);
      setView("editor");
    } catch (e) {
      setError(String(e));
    }
  }

  async function createFolder() {
    if (!currentProjectId) return;
    try {
      await api.createFolder(currentProjectId, name.trim() || "New Folder");
      setFolderOpen(false);
      setName("");
      await selectProject(currentProjectId);
    } catch (e) {
      setError(String(e));
    }
  }

  async function removeProject(id: number) {
    if (!confirm("Delete this project and all prompts?")) return;
    try {
      await api.deleteProject(id);
      await refreshProjects();
    } catch (e) {
      setError(String(e));
    }
  }

  async function removeFile(id: number) {
    if (!confirm("Delete this prompt file?")) return;
    try {
      await api.deletePromptFile(id);
      if (currentProjectId) await selectProject(currentProjectId);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <aside className="flex h-full w-64 flex-col border-r border-[var(--border)] bg-[var(--bg-panel)]/90 backdrop-blur">
      <div className="border-b border-[var(--border)] px-4 py-4">
        <div className="text-xs uppercase tracking-[0.2em] text-[var(--text-muted)]">
          Prompt Git
        </div>
        <div className="mt-1 text-lg font-semibold">Local Prompt VCS</div>
      </div>

      <nav className="flex gap-1 border-b border-[var(--border)] p-2">
        {nav.map((item) => {
          const Icon = item.icon;
          const active = view === item.id;
          return (
            <button
              key={item.id}
              title={item.label}
              onClick={() => setView(item.id)}
              className={`flex flex-1 items-center justify-center rounded-md py-2 ${
                active
                  ? "bg-[var(--accent-soft)] text-[var(--accent)]"
                  : "text-[var(--text-muted)] hover:bg-[var(--bg-hover)]"
              }`}
            >
              <Icon size={16} />
            </button>
          );
        })}
      </nav>

      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
          Projects
        </span>
        <Button variant="ghost" className="!px-2 !py-1" onClick={() => setProjectOpen(true)}>
          <Plus size={14} />
        </Button>
      </div>

      <div className="space-y-1 px-2">
        {projects.map((p) => (
          <div
            key={p.id}
            className={`group flex items-center rounded-md px-2 py-1.5 text-sm ${
              currentProjectId === p.id
                ? "bg-[var(--accent-soft)] text-[var(--accent)]"
                : "hover:bg-[var(--bg-hover)]"
            }`}
          >
            <button
              className="flex-1 truncate text-left"
              onClick={() => selectProject(p.id)}
            >
              {p.name}
            </button>
            <button
              className="hidden text-[var(--text-muted)] group-hover:block hover:text-[var(--danger)]"
              onClick={() => removeProject(p.id)}
            >
              <Trash2 size={12} />
            </button>
          </div>
        ))}
      </div>

      <div className="mt-3 flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
          Files
        </span>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            className="!px-2 !py-1"
            disabled={!currentProjectId}
            onClick={() => setFolderOpen(true)}
          >
            <FolderPlus size={14} />
          </Button>
          <Button
            variant="ghost"
            className="!px-2 !py-1"
            disabled={!currentProjectId}
            onClick={() => setFileOpen(true)}
          >
            <Plus size={14} />
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-auto px-2 pb-4">
        {folders.map((f) => (
          <div
            key={`folder-${f.id}`}
            className="mb-1 rounded-md px-2 py-1 text-xs text-[var(--text-muted)]"
          >
            📁 {f.name}
          </div>
        ))}
        {files.map((f) => (
          <div
            key={f.id}
            className={`group mb-1 flex items-center rounded-md px-2 py-1.5 text-sm ${
              currentFileId === f.id
                ? "bg-[var(--bg-hover)]"
                : "hover:bg-[var(--bg-hover)]/60"
            }`}
          >
            <button
              className="flex flex-1 items-center gap-2 truncate text-left"
              onClick={() => {
                selectFile(f.id);
                setView("editor");
              }}
            >
              <FileText size={14} className="shrink-0 text-[var(--accent)]" />
              <span className="truncate">{f.name}</span>
            </button>
            <button
              className="hidden text-[var(--text-muted)] group-hover:block hover:text-[var(--danger)]"
              onClick={() => removeFile(f.id)}
            >
              <Trash2 size={12} />
            </button>
          </div>
        ))}
        {!files.length && currentProjectId ? (
          <div className="px-2 py-6 text-center text-xs text-[var(--text-muted)]">
            No prompt files yet. Create one to start versioning.
          </div>
        ) : null}
      </div>

      <Modal
        open={projectOpen}
        title="New Project"
        onClose={() => setProjectOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setProjectOpen(false)}>
              Cancel
            </Button>
            <Button onClick={createProject}>Create</Button>
          </>
        }
      >
        <Field label="Name">
          <input className={inputClass} value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Description">
          <input className={inputClass} value={desc} onChange={(e) => setDesc(e.target.value)} />
        </Field>
      </Modal>

      <Modal
        open={fileOpen}
        title="New Prompt File"
        onClose={() => setFileOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setFileOpen(false)}>
              Cancel
            </Button>
            <Button onClick={createFile}>Create</Button>
          </>
        }
      >
        <Field label="File name">
          <input
            className={inputClass}
            placeholder="chat-system.prompt"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
      </Modal>

      <Modal
        open={folderOpen}
        title="New Folder"
        onClose={() => setFolderOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setFolderOpen(false)}>
              Cancel
            </Button>
            <Button onClick={createFolder}>Create</Button>
          </>
        }
      >
        <Field label="Folder name">
          <input className={inputClass} value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
      </Modal>
    </aside>
  );
}
