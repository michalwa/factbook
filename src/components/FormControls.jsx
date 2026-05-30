import styles from "@/styles/FormControls";

export default function FormControls(props) {
  return <div class={styles.controls}>{props.children}</div>;
}
