import { StateEffect } from "@codemirror/state";
import { StateField } from "@codemirror/state";

/** @typedef {{ type: "atom" | "string", name: string, arity: number }} Tag */

/**
 * @type {StateField<Tag[]>}
 */
export const tagCompletions = StateField.define({
  create() {
    return [];
  },
  update(value, tx) {
    for (const effect of tx.effects) {
      if (effect.is(setTagCompletions)) value = effect.value;
    }
    return value;
  },
});

/**
 * @type {StateEffect<Tag[]>}
 */
export const setTagCompletions = StateEffect.define({});

/**
 * @param {import("@codemirror/state").EditorState} state
 *
 * @returns {import("@codemirror/autocomplete").Completion[]}
 */
export function getTagCompletionOptions(state) {
  return state.field(tagCompletions).map(({ type, name, arity }) => {
    name =
      type === "string"
        ? `"${name}"`
        : /^[a-z][A-Za-z0-9_]*$/.test(name)
          ? name
          : `'${name}'`;

    /** @type {import("@codemirror/autocomplete").Completion} */
    const completion = {
      label:
        type === "atom" && arity ? `${name}(${argPlaceholders(arity)})` : name,
      displayLabel: type === "atom" ? `${name}/${arity}` : name,
    };

    return completion;
  });
}

function argPlaceholders(arity) {
  return Array.from({ length: arity }).fill("_").join(", ");
}
