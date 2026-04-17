import { AlertTriangle, CheckCircle2, Info, Loader2, XCircle } from "lucide-react";

export type ActionNoticeTone = "info" | "success" | "warning" | "error" | "running";

export interface ActionNoticeState {
  tone: ActionNoticeTone;
  title: string;
  message: string;
}

interface ActionNoticeProps {
  notice: ActionNoticeState;
  onClose: () => void;
}

export default function ActionNotice({ notice, onClose }: ActionNoticeProps) {
  const Icon =
    notice.tone === "success"
      ? CheckCircle2
      : notice.tone === "warning"
      ? AlertTriangle
      : notice.tone === "error"
      ? XCircle
      : notice.tone === "running"
      ? Loader2
      : Info;
  const toneClass =
    notice.tone === "success"
      ? "border-cyber-green/35 bg-cyber-green/[0.06]"
      : notice.tone === "warning"
      ? "border-cyber-orange/40 bg-cyber-orange/[0.06]"
      : notice.tone === "error"
      ? "border-cyber-red/35 bg-cyber-red/[0.06]"
      : notice.tone === "running"
      ? "border-cyber-cyan/35 bg-cyber-cyan/[0.06]"
      : "border-cyber-blue/35 bg-cyber-blue/[0.06]";
  const iconClass =
    notice.tone === "success"
      ? "text-cyber-green"
      : notice.tone === "warning"
      ? "text-cyber-orange"
      : notice.tone === "error"
      ? "text-cyber-red"
      : notice.tone === "running"
      ? "text-cyber-cyan animate-spin"
      : "text-cyber-blue";

  return (
    <div className={`glass-panel border ${toneClass} p-4`}>
      <div className="flex items-start gap-3">
        <Icon className={`w-4 h-4 mt-0.5 shrink-0 ${iconClass}`} />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold text-cyber-text">{notice.title}</p>
          <p className="mt-1 text-xs text-cyber-muted leading-5">{notice.message}</p>
        </div>
        <button
          className="p-1 rounded text-cyber-muted-dim hover:text-cyber-text hover:bg-cyber-card"
          onClick={onClose}
          title="关闭"
          aria-label="关闭提示"
        >
          <XCircle className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
