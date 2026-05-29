import { createSignal } from "solid-js";

/** @typedef {() => boolean} Toggler */

/**
 * @returns {[Getter<boolean>, Toggler]}
 */
export function createToggle(initial = true) {
  const [get, set] = createSignal(initial);

  /** @type {Toggler} */
  const toggle = () => set(!get());

  return [get, toggle];
}
