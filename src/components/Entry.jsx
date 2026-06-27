import { createEffect, createRoot, createSignal, onCleanup } from "solid-js";
import createCodeEditor from "@/components/CodeEditor";
import styles from "@/styles/Entry";
import { format as formatDate } from "date-fns";
import {
  createEntryLanguageExtension,
  updateSpans,
} from "@/language/entryLanguage";
import { on } from "solid-js";

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

  const [spans, setSpans] = createSignal();
  const { CodeEditor, editorDispatch } = createCodeEditor({
    onSynced: () => setSpans(props.spans),
  });
  const { entryLanguageExtension } = createEntryLanguageExtension();

  createEffect(
    on(spans, (spans) => editorDispatch(updateSpans.of(spans ?? []))),
  );

  return (
    <div class={styles.entry}>
      <time class={styles.timestamp} datetime={formattedTimestamp()}>
        {formattedTimestamp()}
      </time>
      <div class={styles.divider}></div>
      {() => {
        const { editor, dispose } = createRoot((dispose) => {
          return {
            editor: (
              <CodeEditor
                class={styles.content}
                value={props.content}
                onChange={async (content) =>
                  setSpans(await props.parseSpans(content))
                }
                onChangeDeferred={props.onContentChange}
                onEmptyBackspace={props.onRemove}
                extension={entryLanguageExtension()}
              />
            ),
            dispose,
          };
        });

        onCleanup(() => {
          // Delay disposing the editor to prevent height changes ruining the
          // list transition
          queueMicrotask(dispose);
        });

        return editor;
      }}
    </div>
  );
}
