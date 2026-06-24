import { createRoot, createSignal, onCleanup } from "solid-js";
import CodeEditor from "@/components/CodeEditor";
import styles from "@/styles/Entry";
import { format as formatDate } from "date-fns";
import { createEntryLanguageExtension } from "@/language/entryLanguage";

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

  const [spans, setSpans] = createSignal(props.spans);

  const { entryLanguageExtension } = createEntryLanguageExtension();

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
                spans={spans()}
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
