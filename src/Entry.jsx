import { createRoot, onCleanup } from "solid-js";
import CodeEditor from "./CodeEditor";
import styles from "./Entry.module.css";

export default function Entry(props) {
  return (
    <div class={styles.entry}>
      <time class={styles.timestamp} datetime={props.timestamp}>
        {props.timestamp}
      </time>
      <div class={styles.divider}></div>
      {() => {
        const { editor, dispose } = createRoot((dispose) => {
          return {
            editor: <CodeEditor class={styles.content} value={props.content} />,
            dispose,
          };
        });

        // Delay disposing the editor to prevent height changes ruining the list transition
        onCleanup(() => queueMicrotask(dispose));

        return editor;
      }}
    </div>
  );
}
