import styles from "./Entries.module.css";

export default function Entries(props) {
  return <div class={styles.entries}>{props.children}</div>;
}
