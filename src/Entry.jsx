import { createCodeMirror } from "solid-codemirror";
import { EditorView } from "@codemirror/view";
import "./Entry.css";
import { formatDate } from "date-fns";
import { debounce } from "@solid-primitives/scheduled";
import { invoke } from "@tauri-apps/api/core";
import { createEffect } from "solid-js";

export default function Entry(props) {
  const setEntryContent = debounce(
    (content) => invoke("set_entry_content", { entryId: props.id, content }),
    200,
  );

  const {
    ref: editorRef,
    editorView,
    createExtension: createEditorExtension,
  } = createCodeMirror({
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
  createEditorExtension(
    EditorView.domEventHandlers({
      keydown(event) {
        if (event.ctrlKey && ["ArrowUp", "KeyK"].includes(event.code)) {
          props.focusPrev?.();
        } else if (event.ctrlKey && ["ArrowDown", "KeyJ"].includes(event.code)) {
          props.focusNext?.();
        }
      },
    }),
  );

  createEffect(() => {
    if (props.focused && editorView()) editorView().focus();
  });

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
