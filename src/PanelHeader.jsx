import Resizable from "@corvu/resizable";
import "./PanelHeader.css";

export default function PanelHeader(props) {
  const resizable = Resizable.usePanelContext();

  return (
    <div class="panel-header">
      <span class="panel-title">{props.title}</span>
      <button class="icon-button" onClick={() => resizable.collapse()}>
        {props.collapseIcon}
      </button>
    </div>
  );
}
