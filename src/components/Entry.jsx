import { createRoot, createSignal, onCleanup } from "solid-js";
import CodeEditor from "@/components/CodeEditor";
import styles from "@/styles/Entry";
import { format as formatDate } from "date-fns";

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

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
                onChangeDeferred={props.onContentChange}
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
