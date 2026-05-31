import styles from "@/styles/EntriesHeader";

export default function EntriesHeader(props) {
  return <h1 class={styles.header}>{props.children}</h1>;
}
