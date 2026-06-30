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
 * @param {object} detail
 * @param {number} detail.id The id of the entry to foucs
 * @param {"down" | "up"} detail.direction
 *   The direction relative to the previous entry, this will determine which
 *   line the cursor gets set to
 * @param {number | undefined} detail.cursorX
 *   The column in pixels where the cursor should be placed within the focused
 *   entry
 */
export function focusEntry(parentRef, detail) {
  parentRef.dispatchEvent(new CustomEvent(entryFocusRequested, { detail }));
}

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

  const { Editor, dispose: disposeEditor } = createRoot((dispose) => {
    const [spans, setSpans] = createSignal();
    const { entryLanguageExtension } = createEntryLanguageExtension();
    const {
      CodeEditor,
      dispatch,
      focus,
      isCursorAtTop,
      isCursorAtBottom,
      getCursorX,
      moveTo,
    } = createCodeEditor({
      onSynced: () => setSpans(props.spans),
    });

    createEffect(on(spans, (spans) => dispatch(updateSpans.of(spans ?? []))));

    createEventListener(
      () => props.parentRef,
      entryFocusRequested,
      (event) => {
        if (event.detail.id === props.id) {
          focus();
          moveTo({
            line: event.detail.direction === "up" ? "last" : "first",
            cursorX: event.detail.cursorX ?? 0,
          });
        }
      },
    );

    const entryKeymap = createKeymap(props, {
      isCursorAtTop,
      isCursorAtBottom,
      getCursorX,
    });

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

function createKeymap(props, { isCursorAtTop, isCursorAtBottom, getCursorX }) {
  const navigateUp = (view) =>
    props.onNavigateUp?.({
      cursorX: getCursorX(view),
      direction: "up",
    });
  const navigateDown = (view) =>
    props.onNavigateDown?.({
      cursorX: getCursorX(view),
      direction: "down",
    });

  return createMemo(() =>
    keymap.of([
      {
        key: "Backspace",
        run(view) {
          if (view.state.doc.length === 0) props.onRemove?.();
          return true;
        },
      },
      {
        key: "ArrowUp",
        run(view) {
          if (isCursorAtTop(view)) {
            navigateUp(view);
            return true;
          }
        },
      },
      {
        key: "ArrowDown",
        run(view) {
          if (isCursorAtBottom(view)) {
            navigateDown(view);
            return true;
          }
        },
      },
      {
        key: "PageUp",
        run(view) {
          navigateUp(view);
          return true;
        },
      },
      {
        key: "PageDown",
        run(view) {
          navigateDown(view);
          return true;
        },
      },
    ]),
  );
}
