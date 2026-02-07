import { useEffect, useState, useCallback } from "react";
import { MainLayout } from "./components/common/MainLayout";
import { ContentArea } from "./components/common/ContentArea";
import { ComboList } from "./components/combo/ComboList";
import { PreferencesDialog } from "./components/preferences/PreferencesDialog";
import { ImportDialog, ExportDialog, BackupManager } from "./components/data";
import { useGroupStore } from "./stores/groupStore";
import { usePreferencesStore } from "./stores/preferencesStore";
import { useTheme } from "./hooks/useTheme";

function App() {
  const [preferencesOpen, setPreferencesOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [backupsOpen, setBackupsOpen] = useState(false);
  const { selectedGroupId } = useGroupStore();
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

  return (
    <MainLayout
      onOpenPreferences={() => setPreferencesOpen(true)}
      onOpenImport={() => setImportOpen(true)}
      onOpenExport={() => setExportOpen(true)}
      onOpenBackups={() => setBackupsOpen(true)}
    >
      <ContentArea>
        {selectedGroupId ? (
          <ComboList />
        ) : (
          <div className="flex h-full items-center justify-center">
            <h1 className="text-2xl font-bold text-gray-500">
              Select a group to view combos
            </h1>
          </div>
        )}
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
