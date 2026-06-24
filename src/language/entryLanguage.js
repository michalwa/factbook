import { autocompletion } from "@codemirror/autocomplete";
import { getTagCompletionOptions, tagCompletions } from "./autocomplete";

/**
 * @param {import("@codemirror/autocomplete").CompletionContext} context
 *
 * @returns {import("@codemirror/autocomplete").CompletionResult | undefined}
 */
function entryLanguageComplete(context) {
  if (context.state.sliceDoc(context.pos - 1, context.pos) === "@") {
    const options = getTagCompletionOptions(context.state);
    return {
      from: context.pos,
      options,
      validFor: /@'?\w*/,
    };
  }
}

export const entryLanguageExtension = [
  tagCompletions,
  autocompletion({ override: [entryLanguageComplete] }),
];
