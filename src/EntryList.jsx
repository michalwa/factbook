import Entry from "./Entry";
import "./EntryList.css";
import { createEffect, createResource, createSignal, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useViewContext, VIEW_ALL } from "./ViewContext";
import { lookaround, maybe } from "./utils";

export default function EntryList() {
  const [selectedViewId] = useViewContext();
  const [entries, { refetch: refetchEntries }] = createResource(
    selectedViewId,
    (id) => invoke("get_entries", { view: id === VIEW_ALL ? undefined : id }),
  );
  const [selectedEntryId, setSelectedEntryId] = createSignal(null);

  createEffect(() => {
    if (selectedEntryId() === null && entries()?.length)
      setSelectedEntryId(entries()[0].id);
  });

  const createNew = async () => {
    setSelectedEntryId(await invoke("create_entry"));
    refetchEntries();
  };

  const remove = async (id, selectId) => {
    if (selectId) setSelectedEntryId(selectId);
    await invoke("remove_entry", { id });
    refetchEntries();
  };

  return (
    <div class="entry-list">
      <For each={maybe(lookaround, entries())}>
        {([prev, entry, next]) => (
          <Entry
            {...entry}
            focused={selectedEntryId() === entry.id}
            focus={() => setSelectedEntryId(entry.id)}
            focusPrev={() => prev && setSelectedEntryId(prev.id)}
            focusNext={() => next && setSelectedEntryId(next.id)}
            createNew={createNew}
            remove={() => remove(entry.id, next?.id || prev?.id)}
            removeAndFocusPrev={() => prev && remove(entry.id, prev.id)}
            removeAndFocusNext={() => next && remove(entry.id, next.id)}
          />
        )}
      </For>
    </div>
  );
}
