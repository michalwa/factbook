import CodeEditor from "@/components/CodeEditor";
import { createViewLanguageExtension } from "@/language/viewLanguage";

export default function ViewEditor(props) {
  const { viewLanguageExtension } = createViewLanguageExtension();

  return (
    <CodeEditor
      value={props.definition}
      onChangeDeferred={props.onDefinitionChange}
      extension={viewLanguageExtension()}
    />
  );
}
