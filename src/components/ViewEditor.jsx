import CodeEditor from "@/components/CodeEditor";

export default function ViewEditor(props) {
  return (
    <CodeEditor
      value={props.definition}
      onChangeDeferred={props.onDefinitionChange}
    />
  );
}
