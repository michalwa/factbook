import {
  createCodeMirror,
  createEditorControlledValue,
} from "solid-codemirror";
import styles from "@/styles/CodeEditor";
import { EditorView } from "@codemirror/view";
import { debounce } from "@solid-primitives/scheduled";
import { createEffect, createSignal, on, onCleanup, onMount } from "solid-js";
import { StateEffect } from "@codemirror/state";
import { StateField } from "@codemirror/state";
import { Decoration } from "@codemirror/view";

/** @typedef {{ start: number, len: number, kind: string }} Token */

/**
 * @param {object} props
 * @param {(string) => any | undefined} props.onChange
 *   Called immediately every time the editor content changes
 * @param {(string) => any | undefined} props.onChangeDeferred
 *   Called at the trailing edge of a timeout and on cleanup if there are
 *   pending changes
 * @param {number} props.debounce The debounce timeout for `onChangeDeferred`
 * @param {Token[]} props.tokens
 */
export default function CodeEditor(props) {
  // This is the value read back from CodeMirror, not updated with the prop
  const [value, setValue] = createSignal(props.value);
  // Track dirty flag explicitly to be able to conditionally call the deferred
  // callback on cleanup. We don't actually test content for equality, but that's fine
  let dirty = false;
  let initialTokenUpdateDone = false;

  const tokens = () => props.tokens ?? [];

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

  const { ref, editorView, createExtension } = createCodeMirror({
    onValueChange(value) {
      setValue(value);

      if (!initialTokenUpdateDone) {
        editorView()?.dispatch({ effects: updateTokens.of(tokens()) });
        initialTokenUpdateDone = true;
      }
    },
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

  createExtension(tokenHighlight);

  createEffect(
    on(tokens, (tokens) =>
      editorView()?.dispatch({ effects: updateTokens.of(tokens) }),
    ),
  );

  return <div ref={ref} class={`${styles.editor} ${props.class}`} />;
}

/** @type {import("@codemirror/state").StateEffectType<Token[]>} */
const updateTokens = StateEffect.define();

const tokenHighlight = StateField.define({
  create() {
    return Decoration.none;
  },
  update(decorations, transaction) {
    const docLength = transaction.state.doc.length;

    decorations = decorations.map(transaction.changes);

    for (const effect of transaction.effects) {
      if (effect.is(updateTokens)) {
        decorations = Decoration.set(
          effect.value
            .filter(({ start, len }) => start + len <= docLength)
            .map(({ kind, start, len }) =>
              Decoration.mark({ class: `cm-token-${kind}` }).range(
                start,
                start + len,
              ),
            ),
        );
      }
    }

    return decorations;
  },
  provide(field) {
    return EditorView.decorations.from(field);
  },
});
