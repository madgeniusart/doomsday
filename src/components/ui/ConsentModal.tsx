import { Shield, Lock, HardDrive, Eye, ChevronRight } from "lucide-react";
import { useAppStore } from "@/store/appStore";

export function ConsentModal() {
  const giveConsent = useAppStore((s) => s.giveConsent);

  const points = [
    {
      icon: HardDrive,
      title: "Local-first, always",
      body: "All browser data is read and processed on your device. Nothing leaves your machine.",
    },
    {
      icon: Eye,
      title: "Explicit access only",
      body: "We only read browser profiles you explicitly select. Source data is never modified.",
    },
    {
      icon: Lock,
      title: "AES-256-GCM encryption",
      body: "Backups are encrypted with a key stored in your OS keychain (macOS Keychain, Windows DPAPI, Linux Secret Service).",
    },
    {
      icon: Shield,
      title: "Auth cookies stay private",
      body: "Session and authentication cookies are flagged and excluded from all migrations by default.",
    },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-full max-w-lg mx-4 bg-surface-800 border border-white/10 rounded-2xl shadow-2xl animate-slide-in">
        {/* Header */}
        <div className="px-8 pt-8 pb-6 border-b border-white/8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-brand-500/15 flex items-center justify-center">
              <Shield className="w-5 h-5 text-brand-400" />
            </div>
            <div>
              <h1 className="text-lg font-semibold text-white">
                Universal Browser Sync
              </h1>
              <p className="text-xs text-slate-400">Privacy & data access consent</p>
            </div>
          </div>
          <p className="text-sm text-slate-300 leading-relaxed">
            Before accessing any browser data, please review how this application
            handles your information.
          </p>
        </div>

        {/* Privacy points */}
        <div className="px-8 py-6 space-y-4">
          {points.map(({ icon: Icon, title, body }) => (
            <div key={title} className="flex gap-3">
              <div className="w-8 h-8 rounded-lg bg-surface-700 flex items-center justify-center shrink-0 mt-0.5">
                <Icon className="w-4 h-4 text-brand-400" />
              </div>
              <div>
                <p className="text-sm font-medium text-white">{title}</p>
                <p className="text-xs text-slate-400 mt-0.5 leading-relaxed">{body}</p>
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="px-8 pb-8 pt-2">
          <p className="text-xs text-slate-500 mb-4 leading-relaxed">
            By clicking <strong className="text-slate-300">I Understand & Continue</strong>, you
            confirm that you are the owner of the browser profiles you will access, and
            consent to local processing of that data.
          </p>
          <button
            onClick={giveConsent}
            className="btn-primary w-full flex items-center justify-center gap-2 py-3 text-base"
          >
            I Understand & Continue
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
