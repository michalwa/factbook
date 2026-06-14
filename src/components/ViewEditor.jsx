import CodeEditor from "@/components/CodeEditor";
import { createEffect, createSignal } from "solid-js";

export default function ViewEditor(props) {
  const [spans, setSpans] = createSignal(props.spans);

  createEffect(() => setSpans(props.spans));

  return (
    <CodeEditor
      value={props.definition}
      onChange={async (definition) =>
        setSpans(await props.parseSpans(definition))
      }
      onChangeDeferred={props.onDefinitionChange}
      spans={spans()}
    />
  );
}
