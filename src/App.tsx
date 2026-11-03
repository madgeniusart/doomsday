import { useEffect } from "react";
import { useAppStore } from "@/store/appStore";
import { Sidebar } from "@/components/ui/Sidebar";
import { Notification } from "@/components/ui/index";
import { ConsentModal } from "@/components/ui/ConsentModal";
import { Dashboard } from "@/components/browser/Dashboard";
import { MigrationWizard } from "@/components/migration/MigrationWizard";
import { HistoryPanel } from "@/components/migration/HistoryPanel";
import { BackupPanel } from "@/components/settings/BackupPanel";
import { SettingsPanel } from "@/components/settings/SettingsPanel";

export default function App() {
  const {
    view,
    consentGiven,
    notification,
    clearNotification,
    loadSettings,
  } = useAppStore((s) => ({
    view: s.view,
    consentGiven: s.consentGiven,
    notification: s.notification,
    clearNotification: s.clearNotification,
    loadSettings: s.loadSettings,
  }));

  // Load settings on mount to restore consent state
  useEffect(() => {
    loadSettings();
  }, []);

  return (
    <div className="flex h-screen overflow-hidden bg-surface-900 select-none">
      <Sidebar />

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        {view === "dashboard" && <Dashboard />}
        {view === "migrate" && <MigrationWizard />}
        {view === "history" && <HistoryPanel />}
        {view === "backups" && <BackupPanel />}
        {view === "settings" && <SettingsPanel />}
      </main>

      {/* Consent modal (blocks everything until accepted) */}
      {!consentGiven && <ConsentModal />}

      {/* Toast notification */}
      {notification && (
        <div className="fixed bottom-6 right-6 z-50">
          <Notification
            type={notification.type}
            message={notification.message}
            onClose={clearNotification}
          />
        </div>
      )}
    </div>
  );
}
