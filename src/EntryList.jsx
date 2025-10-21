import Entry from "./Entry";
import "./EntryList.css";
import { createResource, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export default function EntryList() {
  const [entries] = createResource(() => invoke("get_all_entries"));

  return (
    <div class="entry-list">
      <For each={entries()}>{(entry) => <Entry {...entry} />}</For>
    </div>
  );
}
