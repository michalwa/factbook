import { createResource, For } from "solid-js";
import "./SidebarViewList.css";
import { invoke } from "@tauri-apps/api/core";
import { useViewContext, VIEW_ALL } from "./ViewContext";

export default function SidebarViewList(props) {
  const [selectedViewId, setSelectedViewId] = useViewContext();
  const [views] = createResource(() => invoke("get_views"));

  const viewsWithAll = () => [{
    id: VIEW_ALL,
    name: "All",
    entryCount: 0,
  }, ...(views() || [])];

  return (
    <ul class="sidebar-view-list">
      <For each={viewsWithAll()}>
        {(view) => (
          <li
            class={`sidebar-view-list-item ${view.id === selectedViewId() ? "selected" : ""}`}
          >
            <a
              href="#"
              class="sidebar-view-list-item-link"
              onClick={() => setSelectedViewId(view.id)}
            >
              {view.name}
              <span class="badge">{view.entryCount}</span>
            </a>
            <Show when={view.id === selectedViewId()}>
              <div class="sidebar-view-list-item-corner-top"></div>
              <div class="sidebar-view-list-item-corner-bottom"></div>
            </Show>
          </li>
        )}
      </For>
    </ul>
  );
}
