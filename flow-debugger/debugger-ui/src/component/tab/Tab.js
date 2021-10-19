import styles from "./Tab.module.css";
import React from "react";

const Tab = ({ focus, label, onClick, ...props }) => {
    return (
        <button
            className={focus ? styles.tabSelected : styles.tab}
            onClick={onClick}
            {...props}
        >
            {label}
        </button>
    );
};

export const TabList = ({ focus, tabs, children = null }) => {
    let tabsLabel = tabs.map((tab, i) => (
        <Tab {...tab} key={i} focus={i === focus} />
    ));
    return (
        <div className={styles.tabs}>
            {tabsLabel}
            {children}
        </div>
    );
};
