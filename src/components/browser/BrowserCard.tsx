import { Circle, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";
import { cn, pluralize } from "@/utils";
import { BROWSER_META, type InstalledBrowser, type BrowserProfile } from "@/types";
import { BrowserIcon } from "@/components/ui/BrowserIcon";
import { Badge } from "@/components/ui/index";

interface BrowserCardProps {
  browser: InstalledBrowser;
  onSelectProfile?: (browser: InstalledBrowser, profile: BrowserProfile) => void;
  selectedProfileId?: string;
  compact?: boolean;
}

export function BrowserCard({
  browser,
  onSelectProfile,
  selectedProfileId,
  compact = false,
}: BrowserCardProps) {
  const [expanded, setExpanded] = useState(false);
  const meta = BROWSER_META[browser.kind];
  const hasProfiles = browser.profiles.length > 0;

  return (
    <div
      className={cn(
        "card border transition-all duration-200",
        selectedProfileId && "border-brand-500/40"
      )}
      style={{ borderColor: selectedProfileId ? undefined : "rgba(255,255,255,0.06)" }}
    >
      {/* Browser header */}
      <div className="p-4 flex items-center gap-3">
        <div
          className="w-10 h-10 rounded-xl flex items-center justify-center shrink-0"
          style={{ background: meta.bgColor }}
        >
          <BrowserIcon kind={browser.kind} size={22} />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold text-white truncate">
              {browser.display_name}
            </span>
            {browser.is_running && (
              <span className="flex items-center gap-1 text-xs text-green-400">
                <Circle className="w-1.5 h-1.5 fill-green-400" />
                Running
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 mt-0.5">
            {browser.version && (
              <span className="text-xs text-slate-500 font-mono">{browser.version.split(" ").slice(-1)[0]}</span>
            )}
            {hasProfiles && (
              <span className="text-xs text-slate-500">
                {pluralize(browser.profiles.length, "profile")}
              </span>
            )}
            {!hasProfiles && (
              <span className="text-xs text-slate-600">No profiles found</span>
            )}
          </div>
        </div>

        {hasProfiles && !compact && (
          <button
            onClick={() => setExpanded((e) => !e)}
            className="btn-ghost p-1.5"
          >
            {expanded ? (
              <ChevronUp className="w-4 h-4" />
            ) : (
              <ChevronDown className="w-4 h-4" />
            )}
          </button>
        )}
      </div>

      {/* Profile list */}
      {(expanded || compact) && hasProfiles && (
        <div className="border-t border-white/6 divide-y divide-white/4">
          {browser.profiles.map((profile) => {
            const isSelected = selectedProfileId === profile.id;
            return (
              <button
                key={profile.id}
                onClick={() => onSelectProfile?.(browser, profile)}
                className={cn(
                  "w-full flex items-center gap-3 px-4 py-3 text-left transition-all duration-150",
                  isSelected
                    ? "bg-brand-600/15"
                    : "hover:bg-white/4"
                )}
              >
                {/* Avatar */}
                <div className="w-7 h-7 rounded-full bg-surface-700 flex items-center justify-center shrink-0 text-xs font-medium text-slate-300">
                  {profile.display_name.slice(0, 1).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-slate-200 truncate">
                      {profile.display_name}
                    </span>
                    {profile.is_default && (
                      <Badge variant="muted">Default</Badge>
                    )}
                  </div>
                  {profile.email && (
                    <p className="text-xs text-slate-500 truncate mt-0.5">
                      {profile.email}
                    </p>
                  )}
                </div>
                {isSelected && (
                  <div className="w-2 h-2 rounded-full bg-brand-400 shrink-0" />
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
