import { createRoot, createSignal, onCleanup } from "solid-js";
import CodeEditor from "@/components/CodeEditor";
import styles from "@/styles/Entry";
import { format as formatDate } from "date-fns";
import { debounce } from "@solid-primitives/scheduled";

export default function Entry(props) {
  const formattedTimestamp = () =>
    formatDate(new Date(props.timestamp), "yyyy-MM-dd hh:mm");

  const [content, setContent] = createSignal(props.content);
  // Track dirty flag explicitly to be able to conditionally call the callback
  // on cleanup. We don't actually test content for equality, but that's fine
  const [dirty, setDirty] = createSignal(false);

  const onContentChange = (content) => {
    if (dirty()) {
      props.onContentChange?.(content);
      setDirty(false);
    }
  };

  // Updating entry content should be fairly cheap, do it as often as reasonably
  // possible to make everything responsive
  const onContentChangeDebounced = debounce(onContentChange, 100);

  const onEditorValueChanged = (value) => {
    setContent(value);
    setDirty(true);
    onContentChangeDebounced(value);
  };

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
                value={content()}
                onChange={onEditorValueChanged}
              />
            ),
            dispose,
          };
        });

        onCleanup(() => {
          // Yield changes if necessary, if the debounced callback did not get a
          // chance to get called
          onContentChange(content());

          // Delay disposing the editor to prevent height changes ruining the
          // list transition
          queueMicrotask(dispose);
        });

        return editor;
      }}
    </div>
  );
}
