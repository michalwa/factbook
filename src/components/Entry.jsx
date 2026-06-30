import {
  createEffect,
  createMemo,
  createRoot,
  createSignal,
  onCleanup,
} from "solid-js";
import createCodeEditor from "@/components/CodeEditor";
import styles from "@/styles/Entry";
import { format as formatDate } from "date-fns";
import {
  createEntryLanguageExtension,
  updateSpans,
} from "@/language/entryLanguage";
import { on } from "solid-js";
import { keymap } from "@codemirror/view";
import { createEventListener } from "@solid-primitives/event-listener";

const entryFocusRequested = "entryFocusRequested";

/**
 * Dispatches an event on the given parent element that will cause a child
 * `Entry` component with the specified entry id to become focused
 *
 * @param {Element} parentRef
 * @param {number} id
 */
export function focusEntry(parentRef, id) {
  parentRef.dispatchEvent(
    new CustomEvent(entryFocusRequested, { detail: { id } }),
  );
}

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

  const { Editor, dispose: disposeEditor } = createRoot((dispose) => {
    const [spans, setSpans] = createSignal();
    const { entryLanguageExtension } = createEntryLanguageExtension();
    const { CodeEditor, dispatch, focus, isCursorAtTop, isCursorAtBottom } =
      createCodeEditor({
        onSynced: () => setSpans(props.spans),
      });

    createEffect(on(spans, (spans) => dispatch(updateSpans.of(spans ?? []))));

    const entryKeymap = createKeymap(props, {
      isCursorAtTop,
      isCursorAtBottom,
    });

    createEventListener(
      () => props.parentRef,
      entryFocusRequested,
      (event) => event.detail.id === props.id && focus(),
    );

    return {
      Editor: () => (
        <CodeEditor
          class={styles.content}
          value={props.content}
          onChange={async (content) =>
            setSpans(await props.parseSpans(content))
          }
          onChangeDeferred={props.onContentChange}
          extension={[entryKeymap(), entryLanguageExtension()]}
        />
      ),
      dispose,
    };
  });

  onCleanup(() => {
    // Delay disposing the editor to prevent height changes ruining the
    // list transition. The timeout should be at least as long as the transition
    // defined in `../styles/TransitionGroup.module.css`
    setTimeout(disposeEditor, 200);
  });

  return (
    <div class={styles.entry}>
      <time class={styles.timestamp} datetime={formattedTimestamp()}>
        {formattedTimestamp()}
      </time>
      <div class={styles.divider}></div>
      <Editor />
    </div>
  );
}

function createKeymap(props, { isCursorAtTop, isCursorAtBottom }) {
  return createMemo(() =>
    keymap.of([
      {
        key: "Backspace",
        run(view) {
          if (view.state.doc.length === 0) props.onRemove?.();
        },
      },
      {
        key: "ArrowUp",
        run(view) {
          if (isCursorAtTop(view)) {
            props.onNavigateUp?.();
            return true;
          }
        },
      },
      {
        key: "ArrowDown",
        run(view) {
          if (isCursorAtBottom(view)) {
            props.onNavigateDown?.();
            return true;
          }
        },
      },
      {
        key: "PageUp",
        run(view) {
          props.onNavigateUp?.();
          return true;
        },
      },
      {
        key: "PageDown",
        run(view) {
          props.onNavigateDown?.();
          return true;
        },
      },
    ]),
  );
}
