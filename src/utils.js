/**
 * @typedef {("shift" | "ctrl" | "alt")} Modifier
 */

/**
 * @param {KeyboardEvent} event
 * @param {Modifier|Modifier[]} modifiers
 * @param {string|string[]} keycodes
 * @returns {boolean}
 */
export function keystroke(event, modifiers, keycodes) {
  return (
    event.shiftKey === (modifiers === "shift" || modifiers.includes("shift")) &&
    event.ctrlKey === (modifiers === "ctrl" || modifiers.includes("ctrl")) &&
    event.altKey === (modifiers === "alt" || modifiers.includes("alt")) &&
    ((typeof keycodes === "string" && event.code === keycodes) ||
      keycodes.includes(event.code))
  );
}
