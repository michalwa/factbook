import { createSignal } from "solid-js";

/** @typedef {() => boolean} Toggler */

/**
 * Creates a signal which can be toggled between two values
 *
 * @returns {[Getter<boolean>, Toggler]}
 */
export function createToggle(firstValue = false, secondValue = true) {
  const [get, set] = createSignal(firstValue);

  /** @type {Toggler} */
  const toggle = () => set(get() === firstValue ? secondValue : firstValue);

  return [get, toggle];
}
