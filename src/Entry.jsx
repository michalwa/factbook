import { createCodeMirror } from "solid-codemirror";
import { EditorView } from "@codemirror/view";
import "./Entry.css";
import { formatDate } from "date-fns";
import { debounce } from "@solid-primitives/scheduled";
import { invoke } from "@tauri-apps/api/core";
import { createEffect, createRoot, createSignal, onCleanup } from "solid-js";
import { keystroke } from "./utils";
import { codeMirrorTheme } from "./codeMirror";

export default function Entry(props) {
  const [dirty, setDirty] = createSignal(false);
  const setContent = async (content) => {
    await invoke("set_entry_content", { id: props.id, content });
    setDirty(false);
  };
  const setContentDebounced = debounce(setContent, 200);

  // Need to create detached root to postpone cleanup until after the list
  // transition has ended, because solid-codemirror destroys the editor in
  // onCleanup which breaks the effect
  const { component, dispose } = createRoot((dispose) => {
    const {
      ref: editorRef,
      editorView,
      createExtension: createEditorExtension,
    } = createCodeMirror({
      value: props.content,
      onValueChange: (content) => {
        setDirty(true);
        setContentDebounced(content);
      },
    });

    createEditorExtension(EditorView.lineWrapping);
    createEditorExtension(codeMirrorTheme);
    createEditorExtension(
      EditorView.domEventHandlers({
        keydown(event, view) {
          const viewEmpty = view.state.doc.length === 0;

          if (keystroke(event, "ctrl", ["ArrowUp", "KeyK"])) {
            props.focusPrev?.();
          } else if (keystroke(event, "ctrl", ["ArrowDown", "KeyJ"])) {
            props.focusNext?.();
          } else if (keystroke(event, "ctrl", "Enter")) {
            (async () => {
              // createNew will unload this component, so we need to skip the debounce
              // and commit the state
              if (dirty()) {
                setContentDebounced.clear();
                await setContent(view.state.doc.toString());
              }

              props.createNew?.();
            })();
          } else if (
            keystroke(event, ["ctrl", "shift"], ["Backspace", "KeyX"])
          ) {
            props.remove?.();
          } else if (keystroke(event, [], "Backspace") && viewEmpty) {
            props.removeAndFocusPrev?.();
          } else if (keystroke(event, [], "Delete") && viewEmpty) {
            props.removeAndFocusNext?.();
          }
        },
        focus() {
          props.focus?.();
        },
      }),
    );

    createEffect(() => {
      if (props.focused && editorView()) editorView().focus();
    });

    const timestamp = () => formatDate(props.createdAt, "yyyy-MM-dd HH:mm");

    return {
      component: (
        <div class="entry">
          <time datetime={timestamp()} class="entry-timestamp">
            {timestamp()}
          </time>
          <div class="entry-divider"></div>
          <div class="entry-content" ref={editorRef}></div>
        </div>
      ),
      dispose,
    };
  });

  // Ensure the timeout is longer than the exit list transition
  onCleanup(() => setTimeout(dispose, 1000));

  return component;
}
