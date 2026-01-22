/**
 * Represents a keyboard shortcut with optional modifier keys
 */
export interface KeyboardShortcut {
  key: string; // "1", "A", "F1", "Space", etc.
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean; // Cmd on macOS, Win on Windows
}

/**
 * Parse a shortcut string into a KeyboardShortcut object
 * @example parseShortcut("Ctrl+Shift+1") => { key: "1", ctrl: true, shift: true, alt: false, meta: false }
 */
export function parseShortcut(str: string): KeyboardShortcut {
  const parts = str.split("+");
  const key = parts[parts.length - 1];

  return {
    key,
    ctrl: parts.some((p) => p.toLowerCase() === "ctrl"),
    alt: parts.some((p) => p.toLowerCase() === "alt"),
    shift: parts.some((p) => p.toLowerCase() === "shift"),
    meta: parts.some(
      (p) => p.toLowerCase() === "meta" || p.toLowerCase() === "cmd",
    ),
  };
}

/**
 * Format a KeyboardShortcut object into a display string
 * @example formatShortcut({ key: "1", ctrl: true, shift: true, ... }) => "Ctrl+Shift+1"
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  if (shortcut.ctrl) parts.push("Ctrl");
  if (shortcut.alt) parts.push("Alt");
  if (shortcut.shift) parts.push("Shift");
  if (shortcut.meta) parts.push("Meta");
  parts.push(shortcut.key);

  return parts.join("+");
}

/**
 * Create a KeyboardShortcut from a KeyboardEvent
 */
export function shortcutFromEvent(event: KeyboardEvent): KeyboardShortcut {
  // Normalize key names
  let key = event.key;

  // Handle special keys
  if (key === " ") key = "Space";
  if (key.length === 1) key = key.toUpperCase();

  return {
    key,
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
  };
}

/**
 * Check if a KeyboardEvent matches a shortcut string
 */
export function eventMatchesShortcut(
  event: KeyboardEvent,
  shortcutStr: string,
): boolean {
  const shortcut = parseShortcut(shortcutStr);

  // Normalize the event key for comparison
  let eventKey = event.key;
  if (eventKey === " ") eventKey = "Space";
  if (eventKey.length === 1) eventKey = eventKey.toUpperCase();

  return (
    eventKey === shortcut.key &&
    event.ctrlKey === shortcut.ctrl &&
    event.altKey === shortcut.alt &&
    event.shiftKey === shortcut.shift &&
    event.metaKey === shortcut.meta
  );
}

/**
 * Check if a key is a modifier key (not a valid shortcut by itself)
 */
export function isModifierKey(key: string): boolean {
  return ["Control", "Alt", "Shift", "Meta", "Command", "Cmd"].includes(key);
}
