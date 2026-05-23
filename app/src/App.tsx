import { useState } from "react";
import { AppShell } from "./components/AppShell";
import { ConfirmDialog, ToastHost } from "./components/Feedback";
import { SetupGuide } from "./components/SetupGuide";
import { Titlebar } from "./components/Titlebar";
import { useModManager } from "./hooks/useModManager";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";
import { LaunchPage } from "./pages/LaunchPage";
import { ModsPage } from "./pages/ModsPage";
import { ProfilesPage } from "./pages/ProfilesPage";
import { SettingsPage } from "./pages/SettingsPage";
import type { PageKey } from "./types/mod";

function App() {
  const manager = useModManager();
  const [currentPage, setCurrentPage] = useState<PageKey>("launch");

  const shellClass = "flex h-screen flex-col overflow-hidden bg-app text-text-primary";

  if (manager.loading) {
    return (
      <div className={shellClass}>
        <Titlebar />
        <div className="grid flex-1 place-items-center">
          <div className="rounded-xl border border-border bg-panel px-8 py-6 shadow-2xl">
            <div className="mx-auto h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent" />
            <div className="mt-4 text-sm text-text-muted">正在读取工作区...</div>
          </div>
        </div>
      </div>
    );
  }

  if (!manager.configured) {
    return (
      <div className={shellClass}>
        <Titlebar />
        <SetupGuide
          busy={manager.busy}
          initialGamePath={manager.gamePath}
          initialMe3Path={manager.me3Path}
          initialLaunchExePath={manager.launchExePath}
          onSetupComplete={manager.savePaths}
        />
        <ToastHost toasts={manager.toasts} />
      </div>
    );
  }

  return (
    <div className={shellClass}>
      <Titlebar onSettingsClick={() => setCurrentPage("settings")} />
      <AppShell
        currentPage={currentPage}
        onPageChange={setCurrentPage}
        activeProfileName={manager.activeProfile?.name ?? "全局启用状态"}
        enabledCount={manager.stats.enabled}
        totalMods={manager.stats.total}
      >
        {currentPage === "launch" && (
          <LaunchPage
            mods={manager.mods}
            activeProfile={manager.activeProfile}
            gamePath={manager.gamePath}
            me3Path={manager.me3Path}
            launchExePath={manager.launchExePath}
            specialModStatus={manager.specialModStatus}
            busy={manager.busy}
            onLaunch={manager.launchGame}
            onRefresh={manager.refresh}
            onOpenDiagnostics={() => setCurrentPage("diagnostics")}
            onPrepareOnline={manager.installSeamlessOnlinefix}
          />
        )}

        {currentPage === "mods" && (
          <ModsPage
            mods={manager.mods}
            activeProfile={manager.activeProfile}
            busy={manager.busy}
            onToggle={manager.toggleMod}
            onDelete={manager.deleteMod}
            onRefresh={manager.refresh}
            onInstallZip={manager.installZip}
            onAddExternalMod={manager.addExternalMod}
            onAddExternalDll={manager.addExternalDll}
            onReadConfig={manager.readModConfig}
            onWriteConfig={manager.writeModConfig}
          />
        )}

        {currentPage === "profiles" && (
          <ProfilesPage
            profiles={manager.profiles}
            activeProfile={manager.activeProfile}
            mods={manager.mods}
            busy={manager.busy}
            onCreate={manager.createProfile}
            onActivate={manager.activateProfile}
            onDelete={manager.deleteProfile}
            onUpdate={manager.updateProfile}
          />
        )}

        {currentPage === "diagnostics" && (
          <DiagnosticsPage
            busy={manager.busy}
            onDiagnose={manager.diagnoseLaunch}
            onGenerateProfile={manager.generateProfilePreview}
            onReadArtifacts={manager.getLaunchArtifacts}
            onDetectConflicts={manager.detectFileConflicts}
            onToast={(message) => manager.pushToast("info", message)}
          />
        )}

        {currentPage === "settings" && (
          <SettingsPage
            gamePath={manager.gamePath}
            me3Path={manager.me3Path}
            launchExePath={manager.launchExePath}
            busy={manager.busy}
            onSave={manager.savePaths}
          />
        )}
      </AppShell>

      <ToastHost toasts={manager.toasts} />
      {manager.confirmState && (
        <ConfirmDialog
          state={manager.confirmState}
          onCancel={() => manager.setConfirmState(null)}
          onConfirm={() => {
            const current = manager.confirmState;
            manager.setConfirmState(null);
            void current?.onConfirm();
          }}
        />
      )}
    </div>
  );
}

export default App;
