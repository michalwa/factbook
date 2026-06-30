import {
  createCodeMirror,
  createEditorControlledValue,
  createEditorFocus,
} from "solid-codemirror";
import styles from "@/styles/CodeEditor";
import { EditorView, keymap } from "@codemirror/view";
import { debounce } from "@solid-primitives/scheduled";
import { createEffect, createSignal, on, onCleanup } from "solid-js";
import { defaultKeymap } from "@codemirror/commands";
import { EditorSelection } from "@codemirror/state";
import { indentWithTab } from "@codemirror/commands";

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

    const { focused } = createEditorFocus(editorView);
    createEffect(
      on(focused, (value) => (value ? props.onFocus?.() : props.onBlur?.())),
    );

    createExtension(EditorView.lineWrapping);
    createExtension(() => props.extension);
    // Register keymap after `props.extension` to allow overrides
    createExtension(keymap.of([defaultKeymap, indentWithTab]));

    return <div ref={ref} class={`${styles.editor} ${props.class}`} />;
  }

  const dispatch = (effects) => editorView()?.dispatch({ effects });
  const focus = () => editorView()?.focus();
  const hasFocus = () => editorView()?.hasFocus;

  const getCursorCoords = (view) =>
    view.coordsAtPos(
      view.state.selection.main.head,
      view.state.selection.main.assoc || undefined,
    );

  /**
   * @param {EditorView | undefined} view
   * @returns {boolean}
   */
  const isCursorAtTop = (view = undefined) => {
    view = view ?? editorView();
    if (!view) return;

    // It's pretty crazy that we have to compare the physical cursor position,
    // I can feel this breaking easily
    const cursor = getCursorCoords(view);
    return cursor.top === view.documentTop;
  };

  /**
   * @param {EditorView | undefined} view
   * @returns {boolean}
   */
  const isCursorAtBottom = (view = undefined) => {
    view = view ?? editorView();
    if (!view) return;

    const cursor = getCursorCoords(view);
    return cursor.bottom === view.documentTop + view.contentHeight;
  };

  /**
   * @param {EditorView | undefined} view
   * @returns {number}
   */
  const getCursorX = (view = undefined) => {
    view = view ?? editorView();
    if (!view) return;

    const cursor = getCursorCoords(view);
    const line = view.state.doc.lineAt(view.state.selection.main.head);
    const lineStart = view.coordsAtPos(line.from);

    return view.state.selection.main.goalColumn ?? cursor.left - lineStart.left;
  };

  /**
   * @param {object} params
   * @param {number | "first" | "last"} params.line The line number (1-based)
   * @param {number} params.cursorX The cursor column in pixels
   */
  const moveTo = ({ line, cursorX }) => {
    const view = editorView();
    if (!view) return;

    const lineNumber =
      line === "first" ? 1 : line === "last" ? view.state.doc.lines : line;
    const docLine = view.state.doc.line(lineNumber);
    const lineStart = view.coordsAtPos(docLine.from);
    const pos = view.posAtCoords({
      x: lineStart.left + cursorX,
      y: lineStart.top,
    });

    view.dispatch({
      selection: EditorSelection.create([
        EditorSelection.cursor(pos, undefined, undefined, cursorX),
      ]),
    });
  };

  return {
    CodeEditor,
    dispatch,
    focus,
    hasFocus,
    isCursorAtTop,
    isCursorAtBottom,
    getCursorX,
    moveTo,
  };
}
