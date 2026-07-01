import createCodeEditor from "@/components/CodeEditor";
import { createViewLanguageExtension } from "@/language/viewLanguage";

export default function createViewEditor() {
  const { CodeEditor, focus, blur, hasFocus } = createCodeEditor();

  const ViewEditor = (props) => {
    const { viewLanguageExtension } = createViewLanguageExtension();

    return (
      <CodeEditor
        value={props.definition}
        onChangeDeferred={props.onDefinitionChange}
        extension={viewLanguageExtension()}
      />
    );
  };

  return { ViewEditor, focus, blur, hasFocus };
}
