import { createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export function useEntries(viewId) {
  const [entries, { refetch: refetchEntries }] = createResource(
    () => ({ viewId: viewId() }), // construct an object to treat `null` as a valid value
    ({ viewId }) => invoke("get_entries", { view: viewId }),
  );

  const setEntryContent = (id, content) =>
    invoke("set_entry_content", { id, content });

  const createEntry = async () => {
    await invoke("create_entry");
    await refetchEntries();
  };

  const removeEntry = async (id) => {
    await invoke("remove_entry", { id });
    await refetchEntries();
  };

  return {
    entries,
    refetchEntries,
    setEntryContent,
    createEntry,
    removeEntry,
  };
}
