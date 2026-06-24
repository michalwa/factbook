import { StateField } from "@codemirror/state";

/** @typedef {{ kind: "atom" | "string", name: string, arity: number }} Tag */

/**
 * @type {StateField<Tag[]>}
 */
export const tagCompletions = StateField.define({
  create() {
    return [];
  },
  update(value, tx) {
    return value;
  },
});

/**
 * @param {import("@codemirror/state").EditorState} state
 *
 * @returns {import("@codemirror/autocomplete").Completion[]}
 */
export function getTagCompletionOptions(state) {
  return state.field(tagCompletions).map(({ kind, name, arity }) => {
    name =
      kind === "string"
        ? `"${name}"`
        : /^[a-z][A-Za-z0-9_]*$/.test(name)
          ? name
          : `'${name}'`;

    /** @type {import("@codemirror/autocomplete").Completion} */
    const completion = {
      label:
        kind === "atom" && arity ? `${name}(${argPlaceholders(arity)})` : name,
      displayLabel: arity ? `${name}/${arity}` : name,
    };

    return completion;
  });
}

function argPlaceholders(arity) {
  return Array.from({ length: arity }).fill("_").join(", ");
}

export const tagCompletionTriggerRegexp = /@['"]?\w*$/;
