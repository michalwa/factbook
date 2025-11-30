import Entry from "./Entry";
import "./EntryList.css";
import { createResource, createSignal, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useViewContext, VIEW_ALL } from "./ViewContext";

export default function EntryList() {
  const [selectedViewId] = useViewContext();
  const [entries] = createResource(selectedViewId, (id) =>
    invoke("get_entries", { view: id === VIEW_ALL ? undefined : id }),
  );

  const [focusedIndex, setFocusedIndex] = createSignal(0);
  const focusPrev = () => {
    if (focusedIndex() > 0) setFocusedIndex(focusedIndex() - 1);
  };
  const focusNext = () => {
    if (focusedIndex() < entries().length - 1)
      setFocusedIndex(focusedIndex() + 1);
  };

  return (
    <div class="entry-list">
      <For each={entries()}>
        {(entry, i) => (
          <Entry
            {...entry}
            focused={focusedIndex() === i()}
            focusPrev={focusPrev}
            focusNext={focusNext}
          />
        )}
      </For>
    </div>
  );
}
