import styles from "@/styles/EntryListHeader";

export default function EntryListHeader(props) {
  return <h1 class={styles.header}>{props.children}</h1>;
}
