import { autocompletion } from "@codemirror/autocomplete";
import {
  getTagCompletionOptions,
  tagCompletions,
  tagCompletionTriggerRegexp,
} from "./autocomplete";
import { useTags } from "@/api/tag";
import { urlHighlight } from "./common";
import { RangeSet, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";

export const updateSpans = StateEffect.define({});

/**
 * Handles decorations returned from the backend entry parser like token
 * highlighting and inline widgets like checkboxes
 */
export const spanDecorations = StateField.define({
  create() {
    return {
      decorations: RangeSet.empty,
      atomicRanges: RangeSet.empty,
    };
  },
  update({ decorations, atomicRanges }, transaction) {
    decorations = decorations.map(transaction.changes);
    atomicRanges = atomicRanges.map(transaction.changes);

    for (const effect of transaction.effects) {
      if (effect.is(updateSpans)) {
        const decorationsArray = [];
        const atomicRangesArray = [];

        for (const span of validSpans(effect.value, transaction)) {
          const decoration = spanDecoration(span);
          decorationsArray.push(decoration);
          if (decoration.value.spec.atomic) atomicRangesArray.push(decoration);
        }

        decorations = RangeSet.of(decorationsArray, true);
        atomicRanges = RangeSet.of(atomicRangesArray, true);
      }
    }

    return { decorations, atomicRanges };
  },
  provide(field) {
    return [
      EditorView.decorations.from(field, ({ decorations }) => decorations),
      EditorView.atomicRanges.from(
        field,
        ({ atomicRanges }) =>
          () =>
            atomicRanges,
      ),
    ];
  },
});

function spanDecoration(span) {
  const { kind, start, len } = span;

  if (kind === "checkbox") {
    return Decoration.replace({
      widget: new CheckboxWidget({ checked: false }),
      // Custom metadata to be able to filter for `EditorView.atomicRanges`
      atomic: true,
    }).range(start, start + len);
  } else {
    return Decoration.mark({ class: `cm-highlight-${kind}` }).range(
      start,
      start + len,
    );
  }
}

class CheckboxWidget extends WidgetType {
  constructor({ checked }) {
    super();
    this.checked = checked;
  }

  eq(other) {
    return this.checked === other.checked;
  }

  toDOM() {
    const wrapper = document.createElement("span");
    wrapper.setAttribute("aria-hidden", "true");
    wrapper.className = "cm-checkbox";

    const checkbox = wrapper.appendChild(document.createElement("input"));
    checkbox.type = "checkbox";
    checkbox.checked = this.checked;

    return wrapper;
  }
}

function validSpans(spans, transaction) {
  const docLength = transaction.state.doc.length;
  return spans.filter(({ start, len }) => start + len <= docLength);
}

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
        spanDecorations,
        urlHighlight,
        tagCompletions.init(() => tagsValue),
        autocompletion({ override: [entryLanguageComplete] }),
      ];
    },
  };
}
