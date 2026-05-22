import { Key } from "@solid-primitives/keyed";
import "./SidebarViewList.css";
import { useViewContext } from "./ViewContext";
import { FunnelPlusIcon } from "lucide-solid";
import TransitionGroup from "./TransitionGroup";

export default function SidebarViewList() {
  const { views, selectedViewId, setSelectedViewId, createView } =
    useViewContext();

  return (
    <>
      <ul class="sidebar-view-list">
        <TransitionGroup>
          <Key each={views()} by="id">
            {(view) => (
              <li class="sidebar-view-list-item">
                <a
                  href="#"
                  class={`sidebar-view-list-item-link ${view().id === selectedViewId() ? "selected" : ""}`}
                  onClick={() => setSelectedViewId(view().id)}
                >
                  {view().name || "Untitled"}
                  <span class="badge">{view().entryCount ?? 0}</span>
                </a>
                <Show when={view().id === selectedViewId()}>
                  <div class="sidebar-view-list-item-corner-top"></div>
                  <div class="sidebar-view-list-item-corner-bottom"></div>
                </Show>
              </li>
            )}
          </Key>
        </TransitionGroup>
      </ul>
      <button class="sidebar-button" onClick={createView}>
        <FunnelPlusIcon size={16} /> New
      </button>
    </>
  );
}
