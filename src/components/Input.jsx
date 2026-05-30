import styles from "@/styles/Input";

export default function Input(props) {
  return <input class={styles.input} type="text" value={props.value ?? ""} />;
}
