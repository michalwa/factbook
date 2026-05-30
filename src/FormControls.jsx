import styles from "./FormControls.module.css";

export default function FormControls(props) {
  return <div class={styles.controls}>{props.children}</div>;
}
