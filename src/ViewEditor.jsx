import Input from "./Input";
import Panel from "./Panel";
import PanelHeader from "./PanelHeader";
import { useViewContext } from "./ViewContext";
import "./ViewEditor.css";
import { PanelBottomCloseIcon } from "lucide-solid";

export default function ViewEditor() {
  const { view, setViewName, viewJustCreated } = useViewContext();

  return (
    <Panel class="view-editor">
      <PanelHeader title="Edit view" collapseIcon={<PanelBottomCloseIcon />} />
      <Input
        value={view()?.name}
        onInput={setViewName}
        focus={viewJustCreated()}
      />
    </Panel>
  );
}
