import Entry from "./Entry";
import "./EntryList.css";
import { createEffect, createResource, createSignal, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useViewContext, VIEW_ALL } from "./ViewContext";

export default function EntryList() {
  const [selectedViewId] = useViewContext();
  const [entries, { refetch: refetchEntries }] = createResource(
    selectedViewId,
    (id) => invoke("get_entries", { view: id === VIEW_ALL ? undefined : id }),
  );
  const [focusedIndex, setFocusedIndex] = createSignal(0);

  createEffect(() => {
    if (entries()) setFocusedIndex(entries().length - 1);
  });

  const focusPrev = () => {
    if (focusedIndex() > 0) setFocusedIndex(focusedIndex() - 1);
  };

  const focusNext = () => {
    if (focusedIndex() < entries().length - 1)
      setFocusedIndex(focusedIndex() + 1);
  };

  const createNew = async () => {
    await invoke("create_entry");
    refetchEntries();
  };

  const removeAt = async (index) => {
    await invoke("remove_entry", { id: entries()[index].id });
    refetchEntries();
  };

  return (
    <div class="entry-list">
      <For each={entries()}>
        {(entry, index) => (
          <Entry
            {...entry}
            focused={focusedIndex() === index()}
            focusPrev={focusPrev}
            focusNext={focusNext}
            createNew={createNew}
            remove={() => removeAt(index())}
          />
        )}
      </For>
    </div>
  );
}
