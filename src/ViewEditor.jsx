import Panel from "./Panel";
import PanelHeader from "./PanelHeader";
import "./ViewEditor.css";
import { PanelBottomCloseIcon, PanelBottomOpenIcon } from "lucide-solid";

export default function ViewEditor() {
  return (
    <Panel class="view-editor">
      <PanelHeader title="Edit view" collapseIcon={<PanelBottomCloseIcon />} />
    </Panel>
  );
}
