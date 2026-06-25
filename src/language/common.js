import { StateField } from "@codemirror/state";
import { StateEffect } from "@codemirror/state";
import { ViewPlugin } from "@codemirror/view";
import { EditorView } from "@codemirror/view";
import { Decoration } from "@codemirror/view";
import { MatchDecorator } from "@codemirror/view";
import { openUrl } from "@tauri-apps/plugin-opener";
import urlRegexSafe from "url-regex-safe";

/** @type {import("@codemirror/state").StateEffectType<Token[]>} */
export const updateSpans = StateEffect.define();

export const spanHighlight = StateField.define({
  create() {
    return Decoration.none;
  },
  update(decorations, transaction) {
    const docLength = transaction.state.doc.length;

    decorations = decorations.map(transaction.changes);

    for (const effect of transaction.effects) {
      if (effect.is(updateSpans)) {
        decorations = Decoration.set(
          effect.value
            .filter(({ start, len }) => start + len <= docLength)
            .map(({ kind, start, len }) =>
              Decoration.mark({ class: `cm-highlight-${kind}` }).range(
                start,
                start + len,
              ),
            ),
        );
      }
    }

    return decorations;
  },
  provide(field) {
    return EditorView.decorations.from(field);
  },
});

const urlDecorator = new MatchDecorator({
  regexp: urlRegexSafe({ strict: true }),
  decoration: (match) => Decoration.mark({ class: "cm-highlight-url" }),
  boundary: /\s/,
});

export const urlHighlight = ViewPlugin.define(
  (view) => ({
    decorations: urlDecorator.createDeco(view),
    update(update) {
      this.decorations = urlDecorator.updateDeco(update, this.decorations);
    },
  }),
  {
    decorations: (value) => value.decorations,
    eventHandlers: {
      // Prevent moving the cursor
      mousedown(event, view) {
        if (
          event.button === 0 &&
          (event.ctrlKey || event.metaKey) &&
          event.target.closest(".cm-highlight-url")
        ) {
          return true;
        }
      },
      click(event, view) {
        if (
          event.button === 0 &&
          (event.ctrlKey || event.metaKey) &&
          event.target.closest(".cm-highlight-url")
        ) {
          const url = event.target.textContent;
          openUrl(url);
          return true;
        }
      },
    },
  },
);
