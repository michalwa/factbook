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

    createExtension(EditorView.lineWrapping);
    createExtension(() => props.extension);
    // Register keymap after `props.extension` to allow overrides
    createExtension(keymap.of(defaultKeymap));

    return <div ref={ref} class={`${styles.editor} ${props.class}`} />;
  }

  const dispatch = (effects) => editorView()?.dispatch({ effects });
  const focus = () => editorView()?.focus();

  /**
   * @param {EditorView | undefined} view
   * @returns {boolean}
   */
  const isCursorAtTop = (view = undefined) => {
    view = view ?? editorView();
    if (!view) return;

    // It's pretty crazy that we have to compare the physical cursor position,
    // I can feel this breaking easily
    const cursor = view.coordsAtPos(
      view.state.selection.main.head,
      view.state.selection.main.assoc,
    );
    return cursor.top === view.documentTop;
  };

  /**
   * @param {EditorView | undefined} view
   * @returns {boolean}
   */
  const isCursorAtBottom = (view = undefined) => {
    view = view ?? editorView().viewportLineBlocks;
    if (!view) return;

    const cursor = view.coordsAtPos(
      view.state.selection.main.head,
      view.state.selection.main.assoc,
    );
    return cursor.bottom === view.documentTop + view.contentHeight;
  };

  return { CodeEditor, dispatch, focus, isCursorAtTop, isCursorAtBottom };
}
