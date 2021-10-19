import React from "react";
import styles from "./Text.module.css";
import { useSelector } from "react-redux";
import { selectTextMessage } from "./TextSlice";

export const Text = () => {
    const message = useSelector(selectTextMessage);
    return (
        <div className={styles.text}>
            <h1> {message} </h1>
        </div>
    );
};
