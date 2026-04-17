import { useEffect, useId, useMemo, useState, type FocusEvent, type KeyboardEvent } from "react";
import { ChevronRight } from "lucide-react";
import type { ProxyConfig } from "../types";
import { ProxyStatusBadge } from "./ProxyStatusBadge";

type ProxySelectProps = {
  proxies: ProxyConfig[];
  selectedProxyId: number | undefined;
  onChange: (proxyId: number | undefined) => void;
  hasWarning?: boolean;
};

type ProxyOption =
  | { type: "direct"; id: "direct"; label: string }
  | { type: "proxy"; id: string; proxy: ProxyConfig; label: string };

export function ProxySelect({ proxies, selectedProxyId, onChange, hasWarning = false }: ProxySelectProps) {
  const reactId = useId();
  const baseId = `proxy-select-${reactId.replace(/:/g, "")}`;
  const listboxId = `${baseId}-listbox`;
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);

  const options = useMemo<ProxyOption[]>(
    () => [
      { type: "direct", id: "direct", label: "不使用代理（直连）" },
      ...proxies.map((proxy) => ({
        type: "proxy" as const,
        id: String(proxy.id),
        proxy,
        label: proxy.name || proxy.url,
      })),
    ],
    [proxies]
  );

  const selectedIndex = selectedProxyId
    ? Math.max(0, proxies.findIndex((proxy) => proxy.id === selectedProxyId) + 1)
    : 0;
  const selectedOption = options[selectedIndex] ?? options[0];

  useEffect(() => {
    setActiveIndex((index) => Math.min(index, options.length - 1));
  }, [options.length]);

  useEffect(() => {
    if (!open) {
      setActiveIndex(selectedIndex);
    }
  }, [open, selectedIndex]);

  const selectIndex = (index: number) => {
    const option = options[index] ?? options[0];
    onChange(option.type === "proxy" ? option.proxy.id : undefined);
    setActiveIndex(index);
    setOpen(false);
  };

  const moveActive = (delta: number) => {
    setOpen(true);
    setActiveIndex((index) => {
      const start = open ? index : selectedIndex;
      return (start + delta + options.length) % options.length;
    });
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveActive(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      moveActive(-1);
    } else if (event.key === "Home") {
      event.preventDefault();
      setOpen(true);
      setActiveIndex(0);
    } else if (event.key === "End") {
      event.preventDefault();
      setOpen(true);
      setActiveIndex(options.length - 1);
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (open) {
        selectIndex(activeIndex);
      } else {
        setActiveIndex(selectedIndex);
        setOpen(true);
      }
    } else if (event.key === "Escape") {
      event.preventDefault();
      setOpen(false);
      setActiveIndex(selectedIndex);
    }
  };

  const handleBlur = (event: FocusEvent<HTMLDivElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
      setOpen(false);
      setActiveIndex(selectedIndex);
    }
  };

  return (
    <div className="relative" onKeyDown={handleKeyDown} onBlur={handleBlur}>
      <button
        type="button"
        role="combobox"
        aria-label={`代理选择：${selectedOption.label}`}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={listboxId}
        aria-activedescendant={open ? `${baseId}-option-${activeIndex}` : undefined}
        className={`cyber-input flex min-h-11 w-full items-center justify-between gap-3 text-left text-sm ${
          hasWarning ? "border-cyber-orange/50 text-cyber-orange" : ""
        }`}
        onClick={() => {
          setActiveIndex(selectedIndex);
          setOpen((value) => !value);
        }}
      >
        <span className="min-w-0 flex-1 truncate">
          {selectedOption.label}
          {selectedOption.type === "proxy" && selectedOption.proxy.username ? (
            <span className="ml-2 text-cyber-muted-dim">[{selectedOption.proxy.username}]</span>
          ) : null}
        </span>
        <span className="flex shrink-0 items-center gap-2">
          {selectedOption.type === "proxy" ? (
            <ProxyStatusBadge status={selectedOption.proxy.status} />
          ) : (
            <DirectBadge />
          )}
          <ChevronRight className={`h-3.5 w-3.5 text-cyber-muted transition-transform ${open ? "rotate-90" : ""}`} />
        </span>
      </button>

      {open && (
        <div
          id={listboxId}
          role="listbox"
          aria-label="代理选项"
          className="absolute z-30 mt-2 max-h-64 w-full overflow-y-auto rounded-md border border-cyber-border bg-black p-1 shadow-[0_18px_42px_rgba(0,0,0,0.45)]"
        >
          {options.map((option, index) => (
            <button
              id={`${baseId}-option-${index}`}
              key={option.id}
              type="button"
              role="option"
              aria-selected={selectedIndex === index}
              className={`flex w-full items-center justify-between gap-3 rounded px-3 py-2 text-left transition-colors ${
                activeIndex === index
                  ? "bg-white/[0.10] text-white"
                  : selectedIndex === index
                  ? "bg-white/[0.06] text-white"
                  : "text-cyber-muted hover:bg-cyber-card hover:text-cyber-text"
              }`}
              onMouseEnter={() => setActiveIndex(index)}
              onClick={() => selectIndex(index)}
            >
              {option.type === "proxy" ? (
                <>
                  <span className="min-w-0">
                    <span className="block truncate text-sm">{option.label}</span>
                    <span className="mt-0.5 block truncate text-[10px] uppercase tracking-wide text-cyber-muted-dim">
                      {option.proxy.proxy_type}
                      {option.proxy.username ? ` / ${option.proxy.username}` : ""}
                    </span>
                  </span>
                  <ProxyStatusBadge status={option.proxy.status} />
                </>
              ) : (
                <>
                  <span className="truncate text-sm">{option.label}</span>
                  <DirectBadge />
                </>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function DirectBadge() {
  return (
    <span className="rounded border border-white/10 bg-white/[0.04] px-2 py-0.5 text-[10px] text-cyber-muted">
      直连
    </span>
  );
}
