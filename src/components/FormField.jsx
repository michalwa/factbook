import styles from "@/styles/FormField";

export default function FormField(props) {
  return <div class={styles.field}>{props.children}</div>;
}
