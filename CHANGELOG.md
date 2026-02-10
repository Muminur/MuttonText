# Changelog

All notable changes to MuttonText will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

---

## [0.0.1] - 2026-02-08

### Added

#### Core Features
- Combo CRUD operations with group management
- Text expansion with strict (word boundary) and loose (suffix) matching modes
- Variable system supporting clipboard, date/time with shifts, cursor positioning, key simulation, input prompts, delay, and recursive combo references
- Combo picker window with relevance-ranked search and keyboard navigation
- Global shortcut system with configurable hotkeys (default: Ctrl+Shift+Space)

#### Platform Support
- Platform abstraction layer for Linux (X11/Wayland) and macOS
- Keyboard hook trait with platform-specific implementations
- Focus detection for Linux (xdotool/xprop) and macOS
- Wayland detection and best-effort support documentation
- macOS accessibility permission checking

#### Import/Export
- Beeftext JSON and CSV import with conflict resolution (skip/overwrite/rename)
- TextExpander CSV import
- Native MuttonText JSON import/export
- TextExpander-compatible CSV export
- Cheatsheet CSV export for quick reference
- Auto-format detection for imported files
- Import preview with combo/group counts

#### Backup & Restore
- Manual and automatic backup creation (.btbackup format)
- Backup restore with data integrity verification
- Backup retention enforcement (configurable max backups)
- Backup management UI with create/restore/delete

#### System Integration
- System tray with state management (active/paused/excluded app)
- Context menu and status tooltip
- Preferences dialog with 6 tabs (Behavior, Appearance, Shortcuts, Data, Updates, Advanced)
- Single instance enforcement via file locking
- Start on login configuration (Linux/macOS)
- Start minimized and close-to-tray behavior
- First-run wizard with 3-step onboarding
- Emoji support with 24 built-in shortcodes and |shortcode| expansion

#### Update System
- Semver-based version comparison
- Update notification in preferences
- Skip version functionality
- Configurable auto-check interval

#### User Interface
- React + TypeScript frontend with Tailwind CSS
- Zustand state management for combos, groups, preferences, and picker
- Main window with sidebar group navigation and combo list
- Combo editor with snippet editor and variable insertion menu
- Syntax highlighting for variable tokens in snippets

#### Accessibility
- Keyboard navigation hooks (focus trap, arrow navigation)
- ARIA labels on all major components (MenuBar, GroupList, ComboList, PickerWindow, PreferencesDialog)
- Screen reader utilities with live region announcements
- WCAG AA color contrast compliance
- Focus-visible outlines and high-contrast mode support
- Reduced motion media query support

#### Performance
- Inline optimizations on hot matching paths with pre-computed keyword lengths
- Keyword-length indexing for O(1) match candidate lookup
- Search result caching with generation-based invalidation
- ComboSummary for lazy loading (lightweight metadata without snippets)
- Memory pooling utilities (PooledBuffer) for allocation reuse
- Compact and memory estimation methods on ComboManager
- Chunked pasting for large snippets (>1000 chars)

#### Packaging
- Tauri v2 build configuration for deb, rpm, AppImage, DMG, NSIS
- AUR PKGBUILD template
- Homebrew cask formula template
- Build and release scripts

### Security
- Environment variable allowlist (14 safe variables only)
- Output size limit (1 MB) and max 100 variables per snippet
- Delay cap (10 seconds) and key count limit (50)
- Backup ID path traversal protection
- Import content size limit (10 MB)
- CSV injection prevention in exports
- Excluded apps list cap (100 entries)
- Preferences numeric bounds validation
- Mutex poisoning recovery in all command handlers
- Input validation on substitution bounds (keyword 256 chars, snippet 100K chars)
- Absolute paths for external commands (security hardening)

### Fixed
- Unsafe unwrap() calls replaced with proper error handling
- Clipboard race conditions handled with retry mechanism (3 retries, 50ms)
- Focus loss during substitution detected via FocusChecker trait
- Mutex poisoning in input manager recovered gracefully

---

[Unreleased]: https://github.com/Muminur/MuttonText/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/Muminur/MuttonText/releases/tag/v0.0.1
