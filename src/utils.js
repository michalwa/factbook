import { createSignal, onCleanup } from "solid-js";

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

export function clickOutside(el, accessor) {
  const onClick = (e) => !el.contains(e.target) && accessor()?.();
  document.body.addEventListener("click", onClick);

  onCleanup(() => document.body.removeEventListener("click", onClick));
}
