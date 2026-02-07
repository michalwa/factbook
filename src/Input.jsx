import "./Input.css";

export default function Input(props) {
  let input;

  return (
    <form action="#" onSubmit={() => input.blur()}>
      <input
        ref={input}
        value={props.value ?? ""}
        onInput={() => props.onInput?.(input.value)}
      />
    </form>
  );
}
