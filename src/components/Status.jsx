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
  Show,
  Switch,
} from "solid-js";
import { useAppState } from "@/api/appState";

export default function Status() {
  const { dirty, lastSaved } = useAppState();

  const [lastSavedString, setLastSavedString] = createSignal();

  onMount(() => {
    const interval = setInterval(() => {
      const lastSavedValue = lastSaved();
      if (lastSavedValue) {
        setLastSavedString(
          formatDistanceToNow(lastSaved(), { addSuffix: true }),
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
      <Show when={lastSaved()}>
        <span
          ref={saveStatusRef}
          class={`${styles.label} ${dirty() ? styles.labelDirty : ""}`}
          title={`last saved ${formatDate(lastSaved(), "yyyy-MM-dd hh:mm:ss")}`}
        >
          <Switch>
            <Match when={dirty()}>
              <Asterisk class={styles.icon} />
              unsaved changes
            </Match>
            <Match when={!dirty()}>
              <Check class={styles.icon} />
              {lastSavedString()}
            </Match>
          </Switch>
        </span>
      </Show>
    </div>
  );
}
