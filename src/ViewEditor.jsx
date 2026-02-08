import {
  createCodeMirror,
  createEditorControlledValue,
} from "solid-codemirror";
import Input from "./Input";
import Panel from "./Panel";
import PanelHeader from "./PanelHeader";
import { useViewContext } from "./ViewContext";
import "./ViewEditor.css";
import { PanelBottomCloseIcon, TrashIcon } from "lucide-solid";
import { EditorView } from "@codemirror/view";
import { codeMirrorTheme } from "./codeMirror";
import { lineNumbers } from "@codemirror/view";

export default function ViewEditor() {
  const { view, setViewName, setViewDefinition, viewJustCreated, removeView } =
    useViewContext();

  const {
    ref: editorRef,
    editorView,
    createExtension: createEditorExtension,
  } = createCodeMirror({
    onValueChange: setViewDefinition,
  });

  createEditorControlledValue(editorView, () => view()?.definition);

  createEditorExtension(EditorView.lineWrapping);
  createEditorExtension(lineNumbers);
  createEditorExtension(codeMirrorTheme);

  return (
    <Panel class="view-editor">
      <PanelHeader title="Edit view" collapseIcon={<PanelBottomCloseIcon />} />
      <Show when={view()}>
        <div class="view-editor-controls">
          <Input
            value={view().name}
            onInput={setViewName}
            onBlur={() => editorView()?.focus()}
            focus={viewJustCreated()}
          />
          <button class="icon-button icon-button-danger" onClick={removeView}>
            <TrashIcon />
          </button>
        </div>
        <div class="view-definition" ref={editorRef}></div>
      </Show>
    </Panel>
  );
}
