import styles from "@/styles/EntriesContainer";

export default function EntriesContainer(props) {
  return (
    <div class={styles.outer}>
      <div class={styles.inner}>{props.children}</div>
      {props.after}
    </div>
  );
}
