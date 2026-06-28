import { load as loadStore } from "@tauri-apps/plugin-store";
import { createResource } from "solid-js";

export function useSettingsStore() {
  const [store] = createResource(() => loadStore("settings.json"));

  const createSetting = (key) => {
    const [value, { mutate, refetch }] = createResource(
      store,
      async (store) => await store?.get(key),
    );
    const set = async (value) => {
      await store()?.set(key, value);
      mutate(value);
    };
    return [value, set, { refetch }];
  };

  return { createSetting };
}
