import styles from "./EntriesContainer.module.css";

export default function EntriesContainer(props) {
  return (
    <div class={styles.outer}>
      <div class={styles.inner}>{props.children}</div>
      {props.after}
    </div>
  );
}
