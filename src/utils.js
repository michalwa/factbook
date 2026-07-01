import { onCleanup } from "solid-js";

/**
 * @template T
 * @param {T[]} array
 * @returns {[T | undefined, T, T | undefined][]}
 */
export function neighbors(array) {
  return (
    array &&
    array.map((_, i) =>
      Array.from({ length: 3 }).map((_, j) => array[i + j - 1]),
    )
  );
}

export function clickOutside(el, accessor) {
  const onClick = (e) => !el.contains(e.target) && accessor()?.();
  document.body.addEventListener("click", onClick);

  onCleanup(() => document.body.removeEventListener("click", onClick));
}
