import { PanelLeftCloseIcon, PanelLeftOpenIcon } from "lucide-solid";
import "./Sidebar.css";
import SidebarViewList from "./SidebarViewList";
import PanelHeader from "./PanelHeader";
import Panel from "./Panel";

export default function Sidebar() {
  return (
    <Panel class="sidebar">
      <PanelHeader title="Views" collapseIcon={<PanelLeftCloseIcon />} />
      <SidebarViewList />
    </Panel>
  );
}
