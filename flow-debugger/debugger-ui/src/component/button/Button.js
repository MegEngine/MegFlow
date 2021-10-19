import React from "react";
import styles from "./Button.module.css";

export const Button = ({ red, bold, icon, rightIcon, children, ...props }) => {
    const c = [styles.container];

    if (bold) {
        c.push(styles.bold);
    }
    if (icon) {
        c.push(styles.hasLeftIcon);
    }
    if (rightIcon) {
        c.push(styles.hasRightIcon);
    }
    if (red) {
        c.push(styles.red);
    }

    return (
        <button className={styles.button} {...props}>
            <div className={c.join(" ")}>
                {icon && <div className={styles.leftIcon}>{icon}</div>}
                {children}
                {rightIcon && (
                    <div className={styles.rightIcon}>{rightIcon}</div>
                )}
            </div>
        </button>
    );
};
