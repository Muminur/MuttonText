import React from "react";
import { MenuBar } from "./MenuBar";
import { Sidebar } from "../group/Sidebar";

interface MainLayoutProps {
  children: React.ReactNode;
  onOpenPreferences?: () => void;
  onOpenImport?: () => void;
  onOpenExport?: () => void;
  onOpenBackups?: () => void;
}

const SIDEBAR_MIN_WIDTH = 180;
const SIDEBAR_MAX_WIDTH = 400;
const SIDEBAR_DEFAULT_WIDTH = 250;
const SIDEBAR_WIDTH_KEY = "muttontext-sidebar-width";

/**
 * Main application layout shell with menu bar, sidebar, and content area.
 * Uses CSS Grid for layout: menu bar at top, sidebar on left, content on right.
 * Sidebar is resizable with min/max width constraints and persists size in localStorage.
 */
export const MainLayout: React.FC<MainLayoutProps> = ({ children, onOpenPreferences, onOpenImport, onOpenExport, onOpenBackups }) => {
  const [sidebarWidth, setSidebarWidth] = React.useState(() => {
    const saved = localStorage.getItem(SIDEBAR_WIDTH_KEY);
    return saved ? parseInt(saved, 10) : SIDEBAR_DEFAULT_WIDTH;
  });

  const [isResizing, setIsResizing] = React.useState(false);

  const handleMouseDown = () => {
    setIsResizing(true);
  };

  React.useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = Math.max(
        SIDEBAR_MIN_WIDTH,
        Math.min(SIDEBAR_MAX_WIDTH, e.clientX)
      );
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      localStorage.setItem(SIDEBAR_WIDTH_KEY, sidebarWidth.toString());
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, sidebarWidth]);

  return (
    <div className="flex h-screen flex-col">
      <MenuBar onOpenPreferences={onOpenPreferences} onOpenImport={onOpenImport} onOpenExport={onOpenExport} onOpenBackups={onOpenBackups} />

      {/* Main content area with resizable sidebar */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar with fixed width */}
        <div
          style={{ width: `${sidebarWidth}px` }}
          className="relative border-r bg-gray-50"
          data-testid="sidebar-slot"
        >
          <Sidebar />

          {/* Resize handle */}
          <div
            className="absolute right-0 top-0 h-full w-1 cursor-col-resize hover:bg-blue-500"
            onMouseDown={handleMouseDown}
            data-testid="sidebar-resize-handle"
          />
        </div>

        {/* Content area */}
        <div className="flex-1 overflow-auto" data-testid="content-slot">
          {children}
        </div>
      </div>
    </div>
  );
};
