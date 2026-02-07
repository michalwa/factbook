import Input from "./Input";
import Panel from "./Panel";
import PanelHeader from "./PanelHeader";
import { useViewContext } from "./ViewContext";
import "./ViewEditor.css";
import { PanelBottomCloseIcon, TrashIcon } from "lucide-solid";

export default function ViewEditor() {
  const { view, setViewName, viewJustCreated, removeView } = useViewContext();

  return (
    <Panel class="view-editor">
      <PanelHeader title="Edit view" collapseIcon={<PanelBottomCloseIcon />} />
      <Show when={view()}>
        <div class="view-editor-controls">
          <Input
            value={view().name}
            onInput={setViewName}
            focus={viewJustCreated()}
          />
          <button class="icon-button icon-button-danger" onClick={removeView}>
            <TrashIcon />
          </button>
        </div>
      </Show>
    </Panel>
  );
}
