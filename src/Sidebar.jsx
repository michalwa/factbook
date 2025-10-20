import { PanelLeftCloseIcon, PanelLeftOpenIcon } from "lucide-solid";
import Resizable from "@corvu/resizable";
import "./Sidebar.css";
import { createSignal, Show } from "solid-js";
import SidebarViewList from "./SidebarViewList";

export default function Sidebar() {
  const resizableContext = Resizable.usePanelContext();
  const [selectedViewId, setSelectedViewId] = createSignal();

  return (
    <div class="sidebar">
      <div class="sidebar-header">
        <span class="sidebar-title">Views</span>
        <button class="icon-button" onClick={() => resizableContext.collapse()}>
          <PanelLeftCloseIcon />
        </button>
        <Show when={resizableContext.collapsed()}>
          <div class="sidebar-collapsed-controls">
            <button
              class="icon-button"
              onClick={() => resizableContext.expand()}
            >
              <PanelLeftOpenIcon />
            </button>
          </div>
        </Show>
      </div>
      <SidebarViewList
        views={[
          { id: 1, name: "All" },
          { id: 2, name: "To do" },
          { id: 3, name: "Work" },
          { id: 4, name: "Recommendations" },
        ]}
        selectedId={selectedViewId()}
        setSelectedId={setSelectedViewId}
      />
    </div>
  );
}
