import styles from "./Badge.module.css";

export default function Badge(props) {
  return (
    <BadgeContainer {...props}>
      <span class={`${styles.badge} ${styles[`size-${props.size}`]}`}>
        {props.children}
      </span>
    </BadgeContainer>
  );
}

function BadgeContainer(props) {
  if (!props.fixedWidth) return props.children;

  return <span class={styles.fixedWidthContainer}>{props.children}</span>;
}
