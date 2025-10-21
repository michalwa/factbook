import { createCodeMirror } from "solid-codemirror";
import { EditorView } from "@codemirror/view";
import "./Entry.css";
import { formatDate } from "date-fns";

export default function Entry(props) {
  const { ref: editorRef, createExtension: createEditorExtension } =
    createCodeMirror({
      value: props.content,
    });

  createEditorExtension(EditorView.lineWrapping);
  createEditorExtension(
    EditorView.theme(
      {
        "&": {
          color: "var(--text-normal)",
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
      },
      { dark: true },
    ),
  );

  const timestamp = () => formatDate(props.timestamp, "yyyy-MM-dd HH:mm");

  return (
    <div class="entry">
      <time datetime={timestamp()} class="entry-timestamp">
        {timestamp()}
      </time>
      <div class="entry-divider"></div>
      <div class="entry-content" ref={editorRef}></div>
    </div>
  );
}
