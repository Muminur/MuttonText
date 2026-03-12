import { useEffect, useState, useCallback } from "react";
import { MainLayout } from "./components/common/MainLayout";
import { ContentArea } from "./components/common/ContentArea";
import { AccessibilityBanner } from "./components/common/AccessibilityBanner";
import { ComboList } from "./components/combo/ComboList";
import { PreferencesDialog } from "./components/preferences/PreferencesDialog";
import { ImportDialog, ExportDialog, BackupManager } from "./components/data";
import { usePreferencesStore } from "./stores/preferencesStore";
import { useTheme } from "./hooks/useTheme";
import { EVENTS } from "./lib/events";

function App() {
  const [preferencesOpen, setPreferencesOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [backupsOpen, setBackupsOpen] = useState(false);
  const { preferences, loadPreferences } = usePreferencesStore();
  useTheme(preferences?.theme);

  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === ",") {
      e.preventDefault();
      setPreferencesOpen(true);
    }
  }, []);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const handleNewCombo = useCallback(() => {
    window.dispatchEvent(new CustomEvent(EVENTS.NEW_COMBO));
  }, []);

  const handleNewGroup = useCallback(() => {
    window.dispatchEvent(new CustomEvent(EVENTS.NEW_GROUP));
  }, []);

  return (
    <MainLayout
      onOpenPreferences={() => setPreferencesOpen(true)}
      onOpenImport={() => setImportOpen(true)}
      onOpenExport={() => setExportOpen(true)}
      onOpenBackups={() => setBackupsOpen(true)}
      onNewCombo={handleNewCombo}
      onNewGroup={handleNewGroup}
    >
      <AccessibilityBanner />
      <ContentArea>
        <ComboList />
      </ContentArea>

      <PreferencesDialog
        isOpen={preferencesOpen}
        onClose={() => setPreferencesOpen(false)}
      />

      <ImportDialog
        isOpen={importOpen}
        onClose={() => setImportOpen(false)}
        onImportComplete={() => {
          // Could trigger a combo refresh here
        }}
      />

      <ExportDialog
        isOpen={exportOpen}
        onClose={() => setExportOpen(false)}
      />

      <BackupManager
        isOpen={backupsOpen}
        onClose={() => setBackupsOpen(false)}
      />
    </MainLayout>
  );
}

export default App;
