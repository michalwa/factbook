import {
  createCodeMirror,
  createEditorControlledValue,
} from "solid-codemirror";
import styles from "@/styles/CodeEditor";
import { EditorView } from "@codemirror/view";
import { debounce } from "@solid-primitives/scheduled";
import { createEffect, createSignal, on, onCleanup } from "solid-js";

/**
 * @param {object} props
 * @param {(string) => any | undefined} props.onChange
 *   Called immediately every time the editor content changes
 * @param {(string) => any | undefined} props.onChangeDeferred
 *   Called at the trailing edge of a timeout and on cleanup if there are
 *   pending changes
 * @param {number} props.debounce The debounce timeout for `onChangeDeferred`
 */
export default function CodeEditor(props) {
  // This is the value read back from CodeMirror, not updated with the prop
  const [value, setValue] = createSignal(props.value);
  // Track dirty flag explicitly to be able to conditionally call the deferred
  // callback on cleanup. We don't actually test content for equality, but that's fine
  const [dirty, setDirty] = createSignal(false);

  const onChangeDeferred = (value) => {
    if (dirty()) {
      props.onChangeDeferred?.(value);
      setDirty(false);
    }
  };
  const onChangeDebounced = debounce(onChangeDeferred, props.debounce ?? 100);

  createEffect(
    on(
      value,
      (v) => {
        setDirty(true);
        props.onChange?.(v);
        onChangeDebounced(v);
      },
      { defer: true },
    ),
  );

  onCleanup(() => onChangeDeferred(value()));

  const { ref, editorView, createExtension } = createCodeMirror({
    onValueChange: setValue,
  });

  // Prevent updating the editor while it's focused
  const [incomingValue, setIncomingValue] = createSignal(props.value);
  createEffect(() => {
    const value = props.value;
    if (!editorView()?.hasFocus) setIncomingValue(value);
  });

  createEditorControlledValue(editorView, incomingValue);

  createExtension(EditorView.lineWrapping);
  createExtension(
    EditorView.domEventHandlers({
      keydown(event, view) {
        if (event.key === "Backspace" && view.state.doc.length === 0)
          props.onEmptyBackspace?.();
      },
    }),
  );

  return <div ref={ref} class={`${styles.editor} ${props.class}`} />;
}
