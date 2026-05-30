import styles from "./Label.module.css";

export default function Label(props) {
  return (
    <Dynamic
      component={props.style === "form" ? "label" : "span"}
      class={`${styles.label} ${props.style && styles[`style-${props.style}`]} ${props.class}`}
    >
      {props.children}
    </Dynamic>
  );
}
