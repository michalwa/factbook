import styles from "./FormField.module.css";

export default function FormField(props) {
  return <div class={styles.field}>{props.children}</div>;
}
