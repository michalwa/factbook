import styles from "./PanelControls.module.css";

export default function PanelControls(props) {
  const placementClasses = () =>
    props.placement
      .split(" ")
      .map((placement) => styles[`placement-${placement}`])
      .join(" ");

  return (
    <div
      class={`${styles.controls} ${placementClasses()} ${styles[`sticky-${props.sticky}`]}`}
    >
      {props.children}
    </div>
  );
}
