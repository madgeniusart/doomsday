import { useEffect } from "react";
import {
  ChevronRight,
  ChevronLeft,
  ArrowRightLeft,
  Play,
  FlaskConical,
} from "lucide-react";
import {
  useAppStore,
  type WizardStep,
  type MigrationWizardState,
} from "@/store/appStore";
import { BrowserCard } from "@/components/browser/BrowserCard";
import { CategorySelector } from "./CategorySelector";
import { ProgressPanel } from "./ProgressPanel";
import { MigrationReport } from "./MigrationReport";
import { BrowserIcon } from "@/components/ui/BrowserIcon";
import { Alert, Spinner } from "@/components/ui/index";
import {
  BROWSER_META,
  type BrowserKind,
  type BrowserProfile,
  type InstalledBrowser,
} from "@/types";
import { cn } from "@/utils";

const STEPS: { id: WizardStep; label: string }[] = [
  { id: "source", label: "Source" },
  { id: "destination", label: "Destination" },
  { id: "categories", label: "Data" },
  { id: "review", label: "Review" },
  { id: "progress", label: "Progress" },
  { id: "report", label: "Report" },
];

const VISIBLE_STEPS = STEPS.slice(0, 4); // steps 5-6 are automatic

export function MigrationWizard() {
  const {
    wizard,
    browsers,
    browsersLoading,
    loadBrowsers,
    setWizardStep,
    setSourceBrowser,
    setDestBrowser,
    toggleCategory,
    selectAllCategories,
    setDryRun,
    startMigration,
  } = useAppStore((s) => ({
    wizard: s.wizard,
    browsers: s.browsers,
    browsersLoading: s.browsersLoading,
    loadBrowsers: s.loadBrowsers,
    setWizardStep: s.setWizardStep,
    setSourceBrowser: s.setSourceBrowser,
    setDestBrowser: s.setDestBrowser,
    toggleCategory: s.toggleCategory,
    selectAllCategories: s.selectAllCategories,
    setDryRun: s.setDryRun,
    startMigration: s.startMigration,
  }));

  useEffect(() => {
    if (browsers.length === 0) loadBrowsers();
  }, []);

  const currentStepIdx = STEPS.findIndex((s) => s.id === wizard.step);
  const isInProgress = wizard.step === "progress" || wizard.step === "report";

  const canNext = () => {
    if (wizard.step === "source") return !!wizard.sourceProfile;
    if (wizard.step === "destination") return !!wizard.destProfile;
    if (wizard.step === "categories") return wizard.selectedCategories.length > 0;
    return true;
  };

  const goNext = () => {
    const next = STEPS[currentStepIdx + 1];
    if (next) setWizardStep(next.id);
  };

  const goPrev = () => {
    const prev = STEPS[currentStepIdx - 1];
    if (prev) setWizardStep(prev.id);
  };

  return (
    <div className="p-8 max-w-3xl mx-auto">
      {/* Wizard step indicator */}
      {!isInProgress && (
        <div className="flex items-center gap-1 mb-8">
          {VISIBLE_STEPS.map((step, i) => {
            const stepIdx = STEPS.findIndex((s) => s.id === step.id);
            const done = stepIdx < currentStepIdx;
            const active = stepIdx === currentStepIdx;
            return (
              <div key={step.id} className="flex items-center gap-1">
                <div
                  className={cn(
                    "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all",
                    active
                      ? "bg-brand-600/20 text-brand-300 border border-brand-500/30"
                      : done
                      ? "text-green-400"
                      : "text-slate-600"
                  )}
                >
                  <span
                    className={cn(
                      "w-4 h-4 rounded-full flex items-center justify-center text-xs font-bold",
                      active ? "bg-brand-500 text-white" : done ? "bg-green-500 text-white" : "bg-surface-700 text-slate-500"
                    )}
                  >
                    {done ? "✓" : i + 1}
                  </span>
                  {step.label}
                </div>
                {i < VISIBLE_STEPS.length - 1 && (
                  <ChevronRight className="w-3 h-3 text-slate-700" />
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Step content */}
      <div className="card p-6">
        {wizard.step === "source" && (
          <SourceStep
            browsers={browsers}
            loading={browsersLoading}
            selectedId={wizard.sourceProfile?.id}
            onSelect={(browser, profile) => {
              setSourceBrowser(browser.kind, profile);
              goNext();
            }}
          />
        )}

        {wizard.step === "destination" && (
          <DestinationStep
            browsers={browsers}
            loading={browsersLoading}
            sourceBrowser={wizard.sourceBrowser}
            sourceProfileId={wizard.sourceProfile?.id}
            selectedId={wizard.destProfile?.id}
            onSelect={(browser, profile) => {
              setDestBrowser(browser.kind, profile);
              goNext();
            }}
          />
        )}

        {wizard.step === "categories" && (
          <CategorySelector
            selected={wizard.selectedCategories}
            onToggle={toggleCategory}
            onSelectAll={selectAllCategories}
          />
        )}

        {wizard.step === "review" && (
          <ReviewStep
            wizard={wizard}
            onDryRunChange={setDryRun}
            onStart={startMigration}
          />
        )}

        {wizard.step === "progress" && <ProgressPanel />}
        {wizard.step === "report" && <MigrationReport />}
      </div>

      {/* Navigation */}
      {!isInProgress && (
        <div className="flex items-center justify-between mt-4">
          <button
            onClick={goPrev}
            disabled={currentStepIdx === 0}
            className="btn-ghost flex items-center gap-1.5 disabled:opacity-0"
          >
            <ChevronLeft className="w-4 h-4" />
            Back
          </button>

          {wizard.step !== "review" && (
            <button
              onClick={goNext}
              disabled={!canNext()}
              className="btn-primary flex items-center gap-2"
            >
              Continue
              <ChevronRight className="w-4 h-4" />
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// ── Step: Source ──────────────────────────────────────────────────────────────

function SourceStep({
  browsers,
  loading,
  selectedId,
  onSelect,
}: {
  browsers: InstalledBrowser[];
  loading: boolean;
  selectedId?: string;
  onSelect: (browser: InstalledBrowser, profile: BrowserProfile) => void;
}) {
  return (
    <div>
      <h2 className="text-lg font-semibold text-white mb-1">Select source browser</h2>
      <p className="text-sm text-slate-400 mb-5">
        Choose the browser you want to migrate data <em>from</em>.
      </p>
      {loading ? (
        <div className="flex justify-center py-10"><Spinner /></div>
      ) : (
        <div className="space-y-3">
          {browsers.map((b) => (
            <BrowserCard
              key={b.kind}
              browser={b}
              compact
              selectedProfileId={selectedId}
              onSelectProfile={onSelect}
            />
          ))}
          {browsers.length === 0 && (
            <p className="text-sm text-slate-500 text-center py-8">
              No browsers found. Install a supported browser first.
            </p>
          )}
        </div>
      )}
    </div>
  );
}

// ── Step: Destination ─────────────────────────────────────────────────────────

function DestinationStep({
  browsers,
  loading,
  sourceBrowser,
  sourceProfileId,
  selectedId,
  onSelect,
}: {
  browsers: InstalledBrowser[];
  loading: boolean;
  sourceBrowser?: BrowserKind;
  sourceProfileId?: string;
  selectedId?: string;
  onSelect: (browser: InstalledBrowser, profile: BrowserProfile) => void;
}) {
  return (
    <div>
      <h2 className="text-lg font-semibold text-white mb-1">Select destination browser</h2>
      <p className="text-sm text-slate-400 mb-5">
        Choose the browser you want to migrate data <em>to</em>.
      </p>

      {sourceBrowser && (
        <Alert type="info">
          Source: <strong>{BROWSER_META[sourceBrowser].name}</strong>. Select a
          different profile below.
        </Alert>
      )}

      <div className="mt-4 space-y-3">
        {loading ? (
          <div className="flex justify-center py-10"><Spinner /></div>
        ) : (
          browsers.map((b) => (
            <BrowserCard
              key={b.kind}
              browser={b}
              compact
              selectedProfileId={selectedId}
              onSelectProfile={(browser, profile) => {
                // Warn if same profile as source
                if (profile.id === sourceProfileId) return;
                onSelect(browser, profile);
              }}
            />
          ))
        )}
      </div>
    </div>
  );
}

// ── Step: Review ──────────────────────────────────────────────────────────────

function ReviewStep({
  wizard,
  onDryRunChange,
  onStart,
}: {
  wizard: MigrationWizardState;
  onDryRunChange: (v: boolean) => void;
  onStart: () => void;
}) {
  const sameBrowser =
    wizard.sourceBrowser === wizard.destBrowser &&
    wizard.sourceProfile?.id === wizard.destProfile?.id;

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold text-white mb-1">Review & start</h2>
        <p className="text-sm text-slate-400">
          Confirm your migration settings before starting.
        </p>
      </div>

      {/* Source → Destination */}
      <div className="flex items-center gap-3 p-4 rounded-xl bg-surface-900/60 border border-white/6">
        <BrowserSummary
          kind={wizard.sourceBrowser}
          name={wizard.sourceProfile?.display_name}
          label="Source"
        />
        <ArrowRightLeft className="w-5 h-5 text-slate-600 shrink-0" />
        <BrowserSummary
          kind={wizard.destBrowser}
          name={wizard.destProfile?.display_name}
          label="Destination"
        />
      </div>

      {/* Data categories */}
      <div className="p-4 rounded-xl bg-surface-900/60 border border-white/6">
        <p className="text-xs text-slate-500 mb-2">Data to migrate</p>
        <div className="flex flex-wrap gap-1.5">
          {wizard.selectedCategories.map((cat: string) => (
            <span
              key={cat}
              className="px-2 py-0.5 rounded-md bg-brand-500/10 text-brand-300 text-xs border border-brand-500/20"
            >
              {cat}
            </span>
          ))}
        </div>
      </div>

      {/* Warnings */}
      {sameBrowser && (
        <Alert type="error">
          Source and destination are the same profile. Please go back and select
          a different destination.
        </Alert>
      )}

      <Alert type="warning">
        Source browser data will not be modified. Exports are written to the
        destination profile folder.
      </Alert>

      {/* Dry run toggle */}
      <label className="flex items-start gap-3 cursor-pointer p-3 rounded-xl border border-white/6 hover:border-white/12 transition-colors">
        <input
          type="checkbox"
          checked={wizard.dryRun}
          onChange={(e) => onDryRunChange(e.target.checked)}
          className="mt-0.5 accent-brand-500"
        />
        <div>
          <p className="text-sm font-medium text-white flex items-center gap-2">
            <FlaskConical className="w-4 h-4 text-brand-400" />
            Dry run (simulate only)
          </p>
          <p className="text-xs text-slate-500 mt-0.5">
            Analyses and reports what would be migrated without writing any
            files.
          </p>
        </div>
      </label>

      {/* Start */}
      <button
        onClick={onStart}
        disabled={sameBrowser || wizard.selectedCategories.length === 0}
        className="btn-primary w-full flex items-center justify-center gap-2 py-3 text-base"
      >
        <Play className="w-4 h-4" />
        {wizard.dryRun ? "Start Dry Run" : "Start Migration"}
      </button>
    </div>
  );
}

function BrowserSummary({
  kind,
  name,
  label,
}: {
  kind?: BrowserKind;
  name?: string;
  label: string;
}) {
  return (
    <div className="flex-1 text-center">
      <p className="text-xs text-slate-500 mb-1">{label}</p>
      {kind ? (
        <>
          <div className="flex justify-center mb-1">
            <BrowserIcon kind={kind} size={28} />
          </div>
          <p className="text-xs font-medium text-white">{BROWSER_META[kind].name}</p>
          {name && <p className="text-xs text-slate-500">{name}</p>}
        </>
      ) : (
        <p className="text-xs text-slate-600">Not selected</p>
      )}
    </div>
  );
}
