import React from "react";
import styles from "./Empty.module.css";

export const Empty = (props) => {
    return <div className={styles.empty}>{props.children}</div>;
};
