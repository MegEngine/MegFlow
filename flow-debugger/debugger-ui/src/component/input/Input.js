import React from "react";
import styles from "./Input.module.css";

export const Input = ({ label, ...props }) => {
    return (
        <div className={styles.container}>
            <label className={styles.inner}> {label} </label>
            <input
                type="text"
                name={label}
                className={styles.inner}
                {...props}
            />
        </div>
    );
};
