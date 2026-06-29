import styles from "@/styles/PanelControls";

export default function PanelControls(props) {
  const placementClasses = () =>
    props.placement
      .split(" ")
      .map((placement) => styles[`placement-${placement}`])
      .join(" ");

  return (
    <div
      class={`${styles.controls} ${placementClasses()} ${styles[`sticky-${props.sticky}`]} ${props.class}`}
    >
      {props.children}
    </div>
  );
}
