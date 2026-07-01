import styles from "@/styles/EntryListContainer";

export default function EntryListContainer(props) {
  return (
    <div class={styles.outer}>
      <div class={styles.inner}>{props.children}</div>
      {props.after}
    </div>
  );
}
