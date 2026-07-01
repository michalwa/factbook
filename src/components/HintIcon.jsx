import { Dynamic } from "solid-js/web";
import styles from "@/styles/HintIcon";

export default function HintIcon(props) {
  return (
    <Dynamic
      component={props.icon}
      class={`${styles.icon} ${styles[`style-${props.style}`]}`}
      title={props.title}
    />
  );
}
