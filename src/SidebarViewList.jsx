import { For } from "solid-js";
import "./SidebarViewList.css";

export default function SidebarViewList(props) {
  return (
    <ul class="sidebar-view-list">
      <For each={props.views}>
        {(view) => (
          <li
            class={`sidebar-view-list-item ${view.id === props.selectedId ? "selected" : ""}`}
          >
            <a
              href="#"
              class="sidebar-view-list-item-link"
              onClick={() => props.setSelectedId(view.id)}
            >
              {view.name}
              <span class="badge">{view.entryCount}</span>
            </a>
            <Show when={view.id === props.selectedId}>
              <div class="sidebar-view-list-item-corner-top"></div>
              <div class="sidebar-view-list-item-corner-bottom"></div>
            </Show>
          </li>
        )}
      </For>
    </ul>
  );
}
