import "./Panel.css";

export default function Panel(props) {
  return <div class={`panel ${props.class}`}>{props.children}</div>;
}
