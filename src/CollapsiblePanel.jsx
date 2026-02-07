import Resizable from "@corvu/resizable";
import "./CollapsiblePanel.css";
import { createSignal } from "solid-js";

export default function CollapsiblePanel(props) {
  const [collapsedControlsEl, setCollapsedControlsEl] = createSignal();

  return (
    <>
      <Resizable.Panel collapsible {...props}>
        {() => {
          const resizable = Resizable.usePanelContext();
          return (
            <>
              {props.children}
              <Show when={resizable.collapsed()}>
                <Portal mount={collapsedControlsEl()}>
                  <button
                    class="icon-button"
                    onClick={() => resizable.expand()}
                  >
                    {props.expandIcon}
                  </button>
                </Portal>
              </Show>
            </>
          );
        }}
      </Resizable.Panel>
      <div
        class="collapsible-panel-controls"
        data-horizontal-align={props.expandButtonHorizontalAlign}
        data-vertical-align={props.expandButtonVerticalAlign}
        ref={setCollapsedControlsEl}
      ></div>
    </>
  );
}
