import type { ProxyConfig } from "../types";

export const proxyStatusLabels = {
  pending: "未检测",
  checking: "检测中",
  available: "可用",
  unavailable: "离线",
  error: "离线",
} satisfies Record<ProxyConfig["status"], string>;

const proxyStatusTone = {
  pending: {
    badge: "border-cyber-muted-dim/30 bg-cyber-muted-dim/10 text-cyber-muted",
    dot: "bg-cyber-muted-dim",
  },
  checking: {
    badge: "border-cyber-cyan/30 bg-cyber-cyan/10 text-cyber-cyan",
    dot: "bg-cyber-cyan animate-pulse",
  },
  available: {
    badge: "border-cyber-green/30 bg-cyber-green/10 text-cyber-green",
    dot: "bg-cyber-green",
  },
  unavailable: {
    badge: "border-cyber-orange/35 bg-cyber-orange/10 text-cyber-orange",
    dot: "bg-cyber-orange",
  },
  error: {
    badge: "border-cyber-orange/35 bg-cyber-orange/10 text-cyber-orange",
    dot: "bg-cyber-orange",
  },
} satisfies Record<ProxyConfig["status"], { badge: string; dot: string }>;

type ProxyStatusBadgeProps = {
  status: ProxyConfig["status"];
  className?: string;
};

export function ProxyStatusBadge({ status, className = "" }: ProxyStatusBadgeProps) {
  const tone = proxyStatusTone[status];
  return (
    <span className={`inline-flex items-center gap-1.5 rounded border px-2 py-0.5 text-[10px] ${tone.badge} ${className}`}>
      <span className={`h-1.5 w-1.5 rounded-full ${tone.dot}`} />
      {proxyStatusLabels[status]}
    </span>
  );
}
