// SnippetEditor - Textarea with syntax highlighting for variables
import { forwardRef, useRef, useEffect, ChangeEvent } from "react";

interface SnippetEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  rows?: number;
  id?: string;
}

// Regex to match variable syntax: #{...}
const VARIABLE_REGEX = /#{[^}]+}/g;

/**
 * Renders text with highlighted variable syntax
 */
function highlightVariables(text: string): React.ReactNode[] {
  const nodes: React.ReactNode[] = [];
  let lastIndex = 0;

  // Find all variable matches
  const matches = Array.from(text.matchAll(VARIABLE_REGEX));

  matches.forEach((match, i) => {
    const matchIndex = match.index!;

    // Add text before the match
    if (matchIndex > lastIndex) {
      nodes.push(
        <span key={`text-${i}`} className="text-transparent">
          {text.substring(lastIndex, matchIndex)}
        </span>
      );
    }

    // Add the highlighted variable
    nodes.push(
      <span key={`var-${i}`} className="text-blue-600 font-semibold">
        {match[0]}
      </span>
    );

    lastIndex = matchIndex + match[0].length;
  });

  // Add remaining text
  if (lastIndex < text.length) {
    nodes.push(
      <span key="text-end" className="text-transparent">
        {text.substring(lastIndex)}
      </span>
    );
  }

  // If no matches, return transparent text
  if (nodes.length === 0) {
    nodes.push(
      <span key="text-all" className="text-transparent">
        {text}
      </span>
    );
  }

  return nodes;
}

export const SnippetEditor = forwardRef<HTMLTextAreaElement, SnippetEditorProps>(
  ({ value, onChange, placeholder, className = "", rows = 6, id }, ref) => {
    const localRef = useRef<HTMLTextAreaElement>(null);
    const overlayRef = useRef<HTMLDivElement>(null);

    // Use external ref or internal ref
    const textareaRef = (ref as React.RefObject<HTMLTextAreaElement>) || localRef;

    // Sync scroll position between textarea and overlay
    useEffect(() => {
      const textarea = textareaRef.current;
      const overlay = overlayRef.current;

      if (!textarea || !overlay) return;

      const syncScroll = () => {
        overlay.scrollTop = textarea.scrollTop;
        overlay.scrollLeft = textarea.scrollLeft;
      };

      textarea.addEventListener("scroll", syncScroll);
      return () => textarea.removeEventListener("scroll", syncScroll);
    }, [textareaRef]);

    const handleChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
      onChange(e.target.value);
    };

    // Base styles shared between textarea and overlay
    const baseStyles = `font-mono text-sm leading-6 px-3 py-2 ${className}`;

    return (
      <div className="relative">
        {/* Highlighting overlay */}
        <div
          ref={overlayRef}
          data-testid="syntax-overlay"
          className={`${baseStyles} absolute inset-0 pointer-events-none overflow-hidden whitespace-pre-wrap break-words border border-transparent rounded-lg`}
          aria-hidden="true"
        >
          {highlightVariables(value)}
          {/* Add trailing newline to match textarea behavior */}
          {value.endsWith("\n") && <span className="text-transparent">{"\n"}</span>}
        </div>

        {/* Actual textarea */}
        <textarea
          ref={textareaRef}
          id={id}
          value={value}
          onChange={handleChange}
          placeholder={placeholder}
          rows={rows}
          className={`${baseStyles} relative bg-transparent caret-black border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none w-full`}
          style={{ color: "transparent", caretColor: "black" }}
          spellCheck={false}
        />
      </div>
    );
  }
);

SnippetEditor.displayName = "SnippetEditor";
