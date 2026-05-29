import styles from "./PanelControlsSpacer.module.css";

export default function PanelControlsSpacer(props) {
  return (
    <div
      class={`${styles.spacer} ${props.when === undefined || props.when ? styles.expanded : styles.collapsed}`}
    ></div>
  );
}
