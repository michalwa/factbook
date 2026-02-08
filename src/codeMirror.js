import { EditorView } from "@codemirror/view";

export const codeMirrorTheme = EditorView.theme(
  {
    "&": {
      color: "var(--text-normal)",
      maxHeight: "100%",
    },
    "&.cm-focused": {
      outline: "none",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "var(--text-normal)",
    },
    "& .cm-selectionBackground": {
      backgroundColor: "var(--bg-selection) !important",
    },
    "& .cm-gutters": {
      background: "none",
      color: "var(--text-dim)"
    },
  },
  { dark: true },
);
