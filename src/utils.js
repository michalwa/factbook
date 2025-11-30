/**
 * Evaluates `f(x)` only if `x` is truthy. Otherwise returns `x`.
 *
 * @template T, U
 * @param {(_: T) => U} f
 * @param {T | null | undefined} x
 * @returns {U | null | undefined}
 */
export function maybe(f, x) {
  return x && f(x);
}

/**
 * Generates an array of sliding windows of length 3 across the input array,
 * where the first and last windows include a trailing `undefined` element,
 * such that for each window the middle element is a subsequent element of
 * the input array: `lookaround(xs)[i][1] === xs[i]`.
 *
 * @template T
 * @param {T[]} xs
 * @returns {[T | undefined, T | undefined, T | undefined][]}
 */
export function lookaround(xs) {
  return xs.map((_, i) => Array.from({ length: 3 }, (_, j) => xs[i + j - 1]));
}

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
