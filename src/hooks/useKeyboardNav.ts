import { useEffect, useCallback, useRef, RefObject } from "react";

/**
 * Traps focus within a container element (e.g., a dialog).
 * When the user tabs past the last focusable element, focus wraps to the first,
 * and vice versa with Shift+Tab.
 */
export function useTrapFocus(containerRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const FOCUSABLE_SELECTOR =
      'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Tab") return;

      const focusableElements = Array.from(
        container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)
      ).filter((el) => el.offsetParent !== null); // visible only

      if (focusableElements.length === 0) return;

      const first = focusableElements[0];
      const last = focusableElements[focusableElements.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    container.addEventListener("keydown", handleKeyDown);
    return () => container.removeEventListener("keydown", handleKeyDown);
  }, [containerRef]);
}

/**
 * Provides arrow key navigation for a list of items.
 * Returns the currently focused index and a keydown handler to attach to the list container.
 *
 * @param itemCount - Total number of items in the list.
 * @param onSelect - Callback invoked when Enter or Space is pressed on a focused item.
 * @param options - Optional configuration: wrap (default true), orientation (default "vertical").
 */
export function useArrowNavigation(
  itemCount: number,
  onSelect: (index: number) => void,
  options: {
    wrap?: boolean;
    orientation?: "vertical" | "horizontal";
    initialIndex?: number;
  } = {}
) {
  const { wrap = true, orientation = "vertical", initialIndex = 0 } = options;
  const focusedIndexRef = useRef(initialIndex);

  const setFocusedIndex = useCallback((index: number) => {
    focusedIndexRef.current = index;
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const prevKey = orientation === "vertical" ? "ArrowUp" : "ArrowLeft";
      const nextKey = orientation === "vertical" ? "ArrowDown" : "ArrowRight";

      if (e.key === nextKey) {
        e.preventDefault();
        let next = focusedIndexRef.current + 1;
        if (next >= itemCount) next = wrap ? 0 : itemCount - 1;
        focusedIndexRef.current = next;

        // Focus the element with matching data-index
        const container = e.currentTarget;
        const target = container.querySelector<HTMLElement>(
          `[data-index="${next}"]`
        );
        target?.focus();
      } else if (e.key === prevKey) {
        e.preventDefault();
        let prev = focusedIndexRef.current - 1;
        if (prev < 0) prev = wrap ? itemCount - 1 : 0;
        focusedIndexRef.current = prev;

        const container = e.currentTarget;
        const target = container.querySelector<HTMLElement>(
          `[data-index="${prev}"]`
        );
        target?.focus();
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        onSelect(focusedIndexRef.current);
      } else if (e.key === "Home") {
        e.preventDefault();
        focusedIndexRef.current = 0;
        const container = e.currentTarget;
        const target = container.querySelector<HTMLElement>(`[data-index="0"]`);
        target?.focus();
      } else if (e.key === "End") {
        e.preventDefault();
        const last = itemCount - 1;
        focusedIndexRef.current = last;
        const container = e.currentTarget;
        const target = container.querySelector<HTMLElement>(
          `[data-index="${last}"]`
        );
        target?.focus();
      }
    },
    [itemCount, onSelect, wrap, orientation]
  );

  return { focusedIndex: focusedIndexRef, setFocusedIndex, handleKeyDown };
}

/**
 * Makes a clickable non-button element keyboard-accessible.
 * Returns onKeyDown handler that triggers onClick for Enter and Space keys.
 */
export function useClickableKeyboard(onClick: () => void) {
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        onClick();
      }
    },
    [onClick]
  );

  return { onKeyDown: handleKeyDown, tabIndex: 0, role: "button" as const };
}
