import { StateField } from "@codemirror/state";

/** @typedef {{
 *   kind: "atom" | "string",
 *   name: string,
 *   count: number
 * } | {
 *   kind: "functor",
 *   name: string,
 *   arity: number,
 *   count: number,
 * }} Tag
 */

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

const tagCompletionTypeMap = Object.freeze({
  atom: "constant",
  functor: "function",
  string: "text",
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
      label:
        kind === "functor" ? `${quoted}(${placeholderArgs(arity)})` : quoted,
      detail: kind === "functor" ? `/${arity}` : undefined,
      type: tagCompletionTypeMap[kind],
      sortText:
        kind === "functor" ? `${name}${String(arity).padStart(3, "0")}` : name,
      boost: count,
    };

    return completion;
  });
}

function placeholderArgs(arity) {
  return Array.from({ length: arity }).fill("_").join(", ");
}

export const tagCompletionTriggerRegexp = /@['"]?\w*$/;
