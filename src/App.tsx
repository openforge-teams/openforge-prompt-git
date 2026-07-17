import { useEffect, useState } from "react";
import { BatchView } from "./components/BatchView";
import { CompareView } from "./components/CompareView";
import { EditorView } from "./components/EditorView";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";
import { Button, Field, inputClass } from "./components/ui";
import { api } from "./lib/api";
import { useAppStore } from "./store";

function UnlockGate() {
  const [password, setPassword] = useState("");
  const setUnlocked = useAppStore((s) => s.setUnlocked);
  const refreshProjects = useAppStore((s) => s.refreshProjects);
  const refreshModels = useAppStore((s) => s.refreshModels);
  const setError = useAppStore((s) => s.setError);

  async function unlock() {
    try {
      const ok = await api.verifyAppPassword(password);
      if (!ok) {
        setError("Incorrect password");
        return;
      }
      setUnlocked(true);
      await refreshProjects();
      await refreshModels();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex h-full items-center justify-center">
      <div className="w-full max-w-sm rounded-2xl border border-[var(--border)] bg-[var(--bg-elevated)] p-6 shadow-2xl">
        <div className="mb-1 text-xs uppercase tracking-[0.2em] text-[var(--text-muted)]">
          Prompt Git
        </div>
        <h1 className="mb-4 text-xl font-semibold">Unlock local workspace</h1>
        <Field label="App password">
          <input
            type="password"
            className={inputClass}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && unlock()}
          />
        </Field>
        <Button className="w-full" onClick={unlock}>
          Unlock
        </Button>
      </div>
    </div>
  );
}

function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const unlocked = useAppStore((s) => s.unlocked);
  const view = useAppStore((s) => s.view);
  const error = useAppStore((s) => s.error);
  const setError = useAppStore((s) => s.setError);
  const loading = useAppStore((s) => s.loading);

  useEffect(() => {
    bootstrap();
  }, [bootstrap]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--text-muted)]">
        Loading Prompt Git...
      </div>
    );
  }

  if (!unlocked) return <UnlockGate />;

  return (
    <div className="flex h-full">
      <Sidebar />
      <main className="relative min-w-0 flex-1">
        {view === "editor" && <EditorView />}
        {view === "compare" && <CompareView />}
        {view === "batch" && <BatchView />}
        {view === "settings" && <SettingsView />}
        {error ? (
          <div className="absolute bottom-4 right-4 max-w-md rounded-lg border border-[var(--danger)] bg-[var(--bg-elevated)] px-4 py-3 text-sm shadow-xl">
            <div className="mb-2 text-[var(--danger)]">{error}</div>
            <Button variant="ghost" onClick={() => setError(null)}>
              Dismiss
            </Button>
          </div>
        ) : null}
      </main>
    </div>
  );
}

export default App;
