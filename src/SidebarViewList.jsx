import { For } from "solid-js";
import "./SidebarViewList.css";
import { useViewContext, VIEW_NEW } from "./ViewContext";
import { FunnelPlusIcon } from "lucide-solid";

export default function SidebarViewList() {
  const { views, selectedViewId, setSelectedViewId, beginCreateView } =
    useViewContext();

  return (
    <>
      <ul class="sidebar-view-list">
        <For each={views()}>
          {(view) => (
            <li
              class={`sidebar-view-list-item ${view.id === selectedViewId() ? "selected" : ""}`}
            >
              <a
                href="#"
                class="sidebar-view-list-item-link"
                onClick={() => setSelectedViewId(view.id)}
              >
                {view.name ?? "Untitled"}
                <Show when={view.id !== VIEW_NEW}>
                  <span class="badge">{view.entryCount ?? 0}</span>
                </Show>
              </a>
              <Show when={view.id === selectedViewId()}>
                <div class="sidebar-view-list-item-corner-top"></div>
                <div class="sidebar-view-list-item-corner-bottom"></div>
              </Show>
            </li>
          )}
        </For>
      </ul>
      <button class="sidebar-button" onClick={beginCreateView}>
        <FunnelPlusIcon size={16} /> New
      </button>
    </>
  );
}
