import { createCodeMirror } from "solid-codemirror";
import { EditorView } from "@codemirror/view";
import "./Entry.css";
import { formatDate } from "date-fns";
import { debounce } from "@solid-primitives/scheduled";
import { invoke } from "@tauri-apps/api/core";

export default function Entry(props) {
  const setEntryContent = debounce(
    (content) => invoke("set_entry_content", { entryId: props.id, content }),
    200,
  );
  const { ref: editorRef, createExtension: createEditorExtension } =
    createCodeMirror({
      value: props.content,
      onValueChange: setEntryContent,
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

  const timestamp = () => formatDate(props.createdAt, "yyyy-MM-dd HH:mm");

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
