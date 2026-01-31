/**
 * Accessibility utilities for MuttonText.
 */

/** Common ARIA labels used throughout the application. */
export const ARIA_LABELS = {
  MENU_BAR: "Main menu",
  FILE_MENU: "File menu",
  EDIT_MENU: "Edit menu",
  COMBOS_MENU: "Combos menu",
  GROUPS_MENU: "Groups menu",
  HELP_MENU: "Help menu",
  SIDEBAR: "Group navigation",
  GROUP_LIST: "Groups",
  COMBO_LIST: "Combos list",
  COMBO_SEARCH: "Search combos",
  PICKER_DIALOG: "Quick combo picker",
  PICKER_SEARCH: "Search for a combo",
  PICKER_RESULTS: "Search results",
  PREFERENCES_DIALOG: "Preferences",
  PREFERENCES_TABS: "Preference categories",
  CLOSE_BUTTON: "Close",
  ADD_GROUP: "Add new group",
} as const;

/**
 * Announces a message to screen readers using a live region.
 * Creates a temporary visually-hidden element with role="status" and aria-live="polite".
 *
 * @param message - The text to announce.
 * @param priority - "polite" (default) or "assertive" for urgent announcements.
 */
export function announceToScreenReader(
  message: string,
  priority: "polite" | "assertive" = "polite"
): void {
  const el = document.createElement("div");
  el.setAttribute("role", "status");
  el.setAttribute("aria-live", priority);
  el.setAttribute("aria-atomic", "true");
  // Visually hidden but accessible to screen readers
  Object.assign(el.style, {
    position: "absolute",
    width: "1px",
    height: "1px",
    padding: "0",
    margin: "-1px",
    overflow: "hidden",
    clip: "rect(0, 0, 0, 0)",
    whiteSpace: "nowrap",
    border: "0",
  });
  document.body.appendChild(el);

  // Set text after a brief delay so the live region is registered first
  requestAnimationFrame(() => {
    el.textContent = message;
  });

  // Clean up after announcement
  setTimeout(() => {
    document.body.removeChild(el);
  }, 1000);
}

/**
 * Returns a set of accessibility props for a given role and label.
 *
 * @param role - The ARIA role to apply.
 * @param label - The accessible label for the element.
 * @param extra - Additional ARIA props to merge.
 */
export function getA11yProps(
  role: string,
  label: string,
  extra: Record<string, string | boolean | number | undefined> = {}
): Record<string, string | boolean | number | undefined> {
  return {
    role,
    "aria-label": label,
    ...extra,
  };
}
