import type { ElementType, ReactNode } from "react";
import { cn } from "@/utils";
import { AlertCircle, CheckCircle2, Info, XCircle, X } from "lucide-react";

// ── Spinner ───────────────────────────────────────────────────────────────────

export function Spinner({ size = "md" }: { size?: "sm" | "md" | "lg" }) {
  return (
    <div
      className={cn(
        "rounded-full border-2 border-white/10 border-t-brand-400 animate-spin",
        size === "sm" && "w-4 h-4",
        size === "md" && "w-6 h-6",
        size === "lg" && "w-10 h-10"
      )}
    />
  );
}

// ── Badge ─────────────────────────────────────────────────────────────────────

type BadgeVariant = "success" | "warning" | "error" | "info" | "muted" | "brand";

export function Badge({
  children,
  variant = "muted",
  className,
}: {
  children: ReactNode;
  variant?: BadgeVariant;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "badge",
        variant === "success" && "badge-success",
        variant === "warning" && "badge-warning",
        variant === "error" && "badge-error",
        variant === "info" && "badge-info",
        variant === "muted" && "badge-muted",
        variant === "brand" && "bg-brand-500/15 text-brand-300",
        className
      )}
    >
      {children}
    </span>
  );
}

// ── Risk badge ────────────────────────────────────────────────────────────────

export function RiskBadge({ level }: { level: string }) {
  const map: Record<string, BadgeVariant> = {
    safe: "success",
    low: "info",
    medium: "warning",
    high: "error",
  };
  return <Badge variant={map[level] ?? "muted"}>{level}</Badge>;
}

// ── Status badge ──────────────────────────────────────────────────────────────

export function StatusBadge({ status }: { status: string }) {
  const map: Record<string, BadgeVariant> = {
    migrated: "success",
    exported: "brand",
    planned: "muted",
    partial: "warning",
    completed: "success",
    failed: "error",
    skipped: "warning",
    incompatible: "warning",
    running: "brand",
    pending: "muted",
    cancelled: "muted",
  };
  return <Badge variant={map[status] ?? "muted"}>{status}</Badge>;
}

// ── Empty state ───────────────────────────────────────────────────────────────

export function Empty({
  icon: Icon,
  title,
  description,
  action,
}: {
  icon?: ElementType;
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      {Icon && (
        <div className="w-12 h-12 rounded-full bg-surface-700 flex items-center justify-center mb-4">
          <Icon className="w-6 h-6 text-slate-500" />
        </div>
      )}
      <h3 className="text-sm font-medium text-slate-300 mb-1">{title}</h3>
      {description && (
        <p className="text-xs text-slate-500 max-w-xs">{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}

// ── Notification toast ────────────────────────────────────────────────────────

export function Notification({
  type,
  message,
  onClose,
}: {
  type: "success" | "error" | "info";
  message: string;
  onClose: () => void;
}) {
  const icons = {
    success: CheckCircle2,
    error: XCircle,
    info: Info,
  };
  const colors = {
    success: "border-green-500/30 bg-green-500/10 text-green-300",
    error: "border-red-500/30 bg-red-500/10 text-red-300",
    info: "border-brand-500/30 bg-brand-500/10 text-brand-300",
  };
  const Icon = icons[type];

  return (
    <div
      className={cn(
        "flex items-start gap-3 px-4 py-3 rounded-xl border text-sm animate-slide-in shadow-xl max-w-sm",
        colors[type]
      )}
    >
      <Icon className="w-4 h-4 mt-0.5 shrink-0" />
      <p className="flex-1 leading-relaxed">{message}</p>
      <button
        onClick={onClose}
        className="text-current opacity-60 hover:opacity-100 transition-opacity mt-0.5"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}

// ── Section header ────────────────────────────────────────────────────────────

export function SectionHeader({
  title,
  description,
  action,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex items-start justify-between mb-6">
      <div>
        <h2 className="text-lg font-semibold text-white">{title}</h2>
        {description && (
          <p className="text-sm text-slate-400 mt-0.5">{description}</p>
        )}
      </div>
      {action && <div>{action}</div>}
    </div>
  );
}

// ── Progress bar ──────────────────────────────────────────────────────────────

export function ProgressBar({
  value,
  className,
  showPercent = false,
}: {
  value: number; // 0–1
  className?: string;
  showPercent?: boolean;
}) {
  const pct = Math.min(100, Math.max(0, value * 100));
  return (
    <div className={cn("flex items-center gap-3", className)}>
      <div className="progress-bar flex-1">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      {showPercent && (
        <span className="text-xs text-slate-400 tabular-nums w-10 text-right">
          {Math.round(pct)}%
        </span>
      )}
    </div>
  );
}

// ── Alert box ─────────────────────────────────────────────────────────────────

export function Alert({
  type = "info",
  children,
}: {
  type?: "info" | "warning" | "error";
  children: ReactNode;
}) {
  const cfg = {
    info: {
      border: "border-brand-500/20",
      bg: "bg-brand-500/8",
      icon: Info,
      iconColor: "text-brand-400",
    },
    warning: {
      border: "border-amber-500/20",
      bg: "bg-amber-500/8",
      icon: AlertCircle,
      iconColor: "text-amber-400",
    },
    error: {
      border: "border-red-500/20",
      bg: "bg-red-500/8",
      icon: XCircle,
      iconColor: "text-red-400",
    },
  }[type];

  const Icon = cfg.icon;

  return (
    <div
      className={cn(
        "flex gap-3 p-3 rounded-lg border text-sm",
        cfg.border,
        cfg.bg
      )}
    >
      <Icon className={cn("w-4 h-4 shrink-0 mt-0.5", cfg.iconColor)} />
      <div className="text-slate-300 leading-relaxed">{children}</div>
    </div>
  );
}
