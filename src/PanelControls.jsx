import styles from "./PanelControls.module.css";

export default function PanelControls(props) {
  return (
    <div
      class={`${styles.controls} ${styles[`placement-${props.placement}`]} ${styles[`sticky-${props.sticky}`]}`}
    >
      {props.children}
    </div>
  );
}
