/**
 * @typedef {("shift" | "ctrl" | "alt")} Modifier
 */

/**
 * Stricly checks for the specified modifiers/keycodes to be present on the
 * given event. Returns `true` iff:
 * * Exactly the modifiers listed in `modifiers` are present on the event.
 *   If any other modifiers are present, returns `false`.
 * * `event.code` is one of the given `keycodes`.
 *
 * Both parameters can also be single strings, in which case they will be
 * handled the same as singleton arrays.
 *
 * @param {KeyboardEvent} event
 * @param {Modifier|Modifier[]} modifiers
 * @param {string|string[]} keycodes
 * @returns {boolean}
 */
export function keystroke(event, modifiers, keycodes) {
  return (
    ["shift", "ctrl", "alt"].every(
      (modifier) =>
        event[`${modifier}Key`] ===
        (modifiers === modifier || modifiers.includes(modifier)),
    ) &&
    ((typeof keycodes === "string" && event.code === keycodes) ||
      keycodes.includes(event.code))
  );
}

export const mapWhere = (filterFn, mapFn) => (xs) =>
  xs.map((x) => (filterFn(x) ? mapFn(x) : x));
