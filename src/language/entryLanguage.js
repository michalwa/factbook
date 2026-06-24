import { autocompletion } from "@codemirror/autocomplete";
import {
  getTagCompletionOptions,
  tagCompletions,
  tagCompletionTriggerRegexp,
} from "./autocomplete";
import { useTags } from "@/api/tag";

/**
 * @param {import("@codemirror/autocomplete").CompletionContext} context
 *
 * @returns {import("@codemirror/autocomplete").CompletionResult | undefined}
 */
function entryLanguageComplete(context) {
  const upToCursor = context.state.sliceDoc(0, context.pos);
  const match = tagCompletionTriggerRegexp.exec(upToCursor);
  if (!match) return;

  const options = getTagCompletionOptions(context.state);

  return {
    from: match.index + 1,
    options,
    validFor: tagCompletionTriggerRegexp,
  };
}

export function createEntryLanguageExtension() {
  const { tags } = useTags();

  return {
    entryLanguageExtension: () => {
      const tagsValue = tags();

      return [
        tagCompletions.init(() => tagsValue),
        autocompletion({ override: [entryLanguageComplete] }),
      ];
    },
  };
}
