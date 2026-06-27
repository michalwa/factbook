import { ViewPlugin, Decoration, MatchDecorator } from "@codemirror/view";
import { openUrl } from "@tauri-apps/plugin-opener";
import urlRegexSafe from "url-regex-safe";

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
