import Entry from "./Entry";
import "./EntryList.css";
import { createResource, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useViewContext, VIEW_ALL } from "./ViewContext";

export default function EntryList() {
  const [selectedViewId] = useViewContext();
  const [entries] = createResource(
    selectedViewId,
    (id) => invoke("get_entries", { view: id === VIEW_ALL ? undefined : id }),
  );

  return (
    <div class="entry-list">
      <For each={entries()}>{(entry) => <Entry {...entry} />}</For>
    </div>
  );
}
