import CodeEditor from "@/components/CodeEditor";
import { viewLanguageExtension } from "@/language/viewLanguage";

export default function ViewEditor(props) {
  return (
    <CodeEditor
      value={props.definition}
      onChangeDeferred={props.onDefinitionChange}
      extension={viewLanguageExtension}
    />
  );
}
