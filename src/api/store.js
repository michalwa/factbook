import { load as loadStore } from "@tauri-apps/plugin-store";
import { createEffect, createResource, createSignal, on } from "solid-js";

export function useJournalSettings({ journalPath }) {
  const [store] = createResource(() => loadStore("settings.json"));

  const createJournalSetting = (key) => {
    const [lastStored] = createResource(
      () => [store(), journalPath()],
      async ([store, path]) =>
        path && (await store?.get("journal_settings"))?.[path]?.[key],
    );

    const [get, set] = createSignal();
    createEffect(() => set(lastStored()));
    createEffect(
      on(get, async (value) => {
        const path = journalPath();
        if (!path) return;

        const allSettings = await store().get("journal_settings");
        await store().set("journal_settings", {
          ...allSettings,
          [path]: {
            ...allSettings[path],
            [key]: value,
          },
        });
      }),
    );

    return [get, set];
  };

  return { createJournalSetting };
}
