import { clsx } from "clsx";
import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

export function Button({
  children,
  variant = "primary",
  className,
  ...props
}: PropsWithChildren<
  ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: "primary" | "ghost" | "danger" | "secondary";
  }
>) {
  return (
    <button
      className={clsx(
        "inline-flex items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition disabled:opacity-50",
        variant === "primary" &&
          "bg-[var(--accent)] text-white hover:brightness-110",
        variant === "secondary" &&
          "bg-[var(--bg-hover)] text-[var(--text)] border border-[var(--border)] hover:bg-[var(--border)]",
        variant === "ghost" &&
          "bg-transparent text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]",
        variant === "danger" &&
          "bg-[var(--danger)] text-white hover:brightness-110",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}

export function Modal({
  open,
  title,
  onClose,
  children,
  footer,
}: PropsWithChildren<{
  open: boolean;
  title: string;
  onClose: () => void;
  footer?: React.ReactNode;
}>) {
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-lg rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-4 py-3">
          <h3 className="text-base font-semibold">{title}</h3>
          <Button variant="ghost" onClick={onClose}>
            Close
          </Button>
        </div>
        <div className="px-4 py-4">{children}</div>
        {footer ? (
          <div className="flex justify-end gap-2 border-t border-[var(--border)] px-4 py-3">
            {footer}
          </div>
        ) : null}
      </div>
    </div>
  );
}

export function Field({
  label,
  children,
}: PropsWithChildren<{ label: string }>) {
  return (
    <label className="mb-3 block text-sm">
      <span className="mb-1 block text-[var(--text-muted)]">{label}</span>
      {children}
    </label>
  );
}

export const inputClass =
  "w-full rounded-md border border-[var(--border)] bg-[var(--editor)] px-3 py-2 text-sm outline-none focus:border-[var(--accent)]";

export const textareaClass =
  "w-full rounded-md border border-[var(--border)] bg-[var(--editor)] px-3 py-2 text-sm outline-none focus:border-[var(--accent)] mono min-h-[120px] resize-y";
