import styles from "@/styles/Status";
import { Asterisk, Check } from "lucide-solid";
import { formatDate, formatDistanceToNow } from "date-fns";
import {
  createEffect,
  createSignal,
  Match,
  on,
  onCleanup,
  onMount,
  Switch,
} from "solid-js";
import { useAppState } from "@/api/appState";

export default function Status() {
  const { dirty, lastSaved } = useAppState();

  const [lastSavedString, setLastSavedString] = createSignal();
  const [lastSavedDetailedString, setLastSavedDetailedString] = createSignal();

  onMount(() => {
    const interval = setInterval(() => {
      const lastSavedValue = lastSaved();
      if (lastSavedValue) {
        setLastSavedString(
          formatDistanceToNow(lastSavedValue, { addSuffix: true }),
        );
        setLastSavedDetailedString(
          formatDate(lastSavedValue, "yyyy-MM-dd hh:mm:ss"),
        );
      }
    });
    onCleanup(() => clearInterval(interval));
  });

  let saveStatusRef;

  createEffect(
    on(lastSaved, () => {
      saveStatusRef?.classList.remove(styles.labelClean);
      if (!dirty()) {
        void saveStatusRef?.offsetWidth; // Force reflow
        saveStatusRef?.classList.add(styles.labelClean);
      }
    }),
  );

  return (
    <div class={styles.container}>
      <span
        ref={saveStatusRef}
        class={`${styles.label} ${dirty() ? styles.labelDirty : ""}`}
        title={
          lastSavedDetailedString()
            ? `last saved ${lastSavedDetailedString()}`
            : ""
        }
      >
        <Switch>
          <Match when={dirty()}>
            <Asterisk class={styles.icon} />
            unsaved changes
          </Match>
          <Match when={!dirty() && lastSaved()}>
            <Check class={styles.icon} />
            {lastSavedString()}
          </Match>
        </Switch>
      </span>
    </div>
  );
}
