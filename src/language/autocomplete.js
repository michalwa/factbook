import { StateField } from "@codemirror/state";

/** @typedef {{ kind: "atom" | "string", name: string, arity: number, count: number }} Tag */

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
  return state.field(tagCompletions).map(({ kind, name, arity, count }) => {
    const quoted =
      kind === "string"
        ? `"${name}"`
        : /^[a-z][A-Za-z0-9_]*$/.test(name)
          ? name
          : `'${name}'`;

    /** @type {import("@codemirror/autocomplete").Completion} */
    const completion = {
      displayLabel: quoted,
      label: `${quoted}${placeholderArgs(arity)}`,
      info: `${quoted}${placeholderArgs(arity)}`,
      detail: arity && `/${arity}`,
      type: kind === "atom" ? (arity ? "function" : "constant") : "text",
      sortText: `${name}${String(arity).padStart(3, "0")}`,
      boost: count,
    };

    return completion;
  });
}

function placeholderArgs(arity) {
  return arity ? `(${Array.from({ length: arity }).fill("_").join(", ")})` : "";
}

export const tagCompletionTriggerRegexp = /@['"]?\w*$/;
