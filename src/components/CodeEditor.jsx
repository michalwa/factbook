import {
  createCodeMirror,
  createEditorControlledValue,
} from "solid-codemirror";
import styles from "@/styles/CodeEditor";
import { EditorView, keymap } from "@codemirror/view";
import { debounce } from "@solid-primitives/scheduled";
import { createEffect, createSignal, on, onCleanup } from "solid-js";
import { defaultKeymap } from "@codemirror/commands";

/**
 * @param {object} config
 * @param {() => any} config.onSynced Called after the editor has been initially
 *   populated with content provided by the `value` prop
 */
export default function createCodeEditor(config = {}) {
  // This is the value read back from CodeMirror, not updated with the prop
  const [value, setValue] = createSignal();
  let synced = false;

  const { ref, editorView, createExtension } = createCodeMirror({
    onValueChange(value) {
      // First change event is fired after the editor content is set to the prop
      // value, we don't want to fire a normal change event yet
      if (!synced) {
        synced = true;
        config.onSynced?.();
      } else {
        setValue(value);
      }
    },
  });

  function CodeEditor(props) {
    setValue(props.value);

    // Track dirty flag explicitly to be able to conditionally call the deferred
    // callback on cleanup. We don't actually test content for equality, but that's fine
    let dirty = false;

    const onChangeDeferred = (value) => {
      if (dirty) {
        props.onChangeDeferred?.(value);
        dirty = false;
      }
    };
    const onChangeDebounced = debounce(onChangeDeferred, props.debounce ?? 100);

    createEffect(
      on(
        value,
        (v) => {
          dirty = true;
          props.onChange?.(v);
          onChangeDebounced(v);
        },
        { defer: true },
      ),
    );

    onCleanup(() => onChangeDeferred(value()));

    // Create indirection between prop and editor to prevent updating the editor
    // while it's focused
    const [incomingValue, setIncomingValue] = createSignal(props.value);
    createEffect(
      on(
        () => props.value,
        (v) => {
          // Need to manually compare, because `props.value` seems to trigger
          // reactivity even if the new value equals the old value
          if (incomingValue() !== v && !editorView()?.hasFocus) {
            synced = false;
            setIncomingValue(v);
          }
        },
      ),
    );

    createEditorControlledValue(editorView, incomingValue);

    createExtension([
      EditorView.lineWrapping,
      EditorView.domEventHandlers({
        keydown(event, view) {
          if (event.key === "Backspace" && view.state.doc.length === 0)
            props.onEmptyBackspace?.();
        },
      }),
      keymap.of(defaultKeymap),
    ]);

    createExtension(() => props.extension);

    return <div ref={ref} class={`${styles.editor} ${props.class}`} />;
  }

  const editorDispatch = (effects) => editorView()?.dispatch({ effects });

  return { CodeEditor, editorDispatch };
}
