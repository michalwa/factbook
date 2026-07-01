import { createEffect, createResource, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useAppState } from "@/api/appState";

export function useEntries(viewId) {
  const { setDirty } = useAppState();

  const [rawEntries, { refetch: refetchEntries, mutate: mutateEntries }] =
    createResource(
      () => ({ viewId: viewId() }), // construct an object to treat `null` as a valid value
      ({ viewId }) => invoke("get_entries", { view: viewId }),
    );
  const [entries, setEntries] = createSignal();
  const [viewError, setViewError] = createSignal();

  createEffect(() => {
    const entriesValue = rawEntries();
    setViewError(entriesValue?.error);
    if (!entriesValue?.error) setEntries(entriesValue);
  });

  const setEntryContent = async (id, content) => {
    setDirty(true);
    return await invoke("set_entry_content", { id, content });
  };

  const parseEntryContent = (content) =>
    invoke("parse_entry_content", { content });

  const createEntry = async () => {
    setDirty(true);
    const id = await invoke("create_entry");
    mutateEntries((entries) => [...entries, { id, createdAt: new Date() }]);
    return id;
  };

  const removeEntry = async (id) => {
    setDirty(true);
    await invoke("remove_entry", { id });
    mutateEntries((entries) => entries.filter((entry) => entry.id !== id));
  };

  return {
    entries,
    refetchEntries,
    setEntryContent,
    parseEntryContent,
    createEntry,
    removeEntry,
    viewError,
  };
}
