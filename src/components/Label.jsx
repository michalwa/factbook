import styles from "@/styles/Label";

export default function Label(props) {
  return (
    <label
      class={`${styles.label} ${props.style && styles[`style-${props.style}`]} ${props.class}`}
    >
      {props.children}
    </label>
  );
}
