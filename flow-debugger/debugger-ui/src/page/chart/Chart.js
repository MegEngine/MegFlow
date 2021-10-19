import React from "react";
import styles from "./Chart.module.css";
import { useSelector } from "react-redux";
import { LineWithDescp } from "../../component/line_with_descp/LineWithDescp";
import { selectChartLines, selectChartFocus } from "./ChartSlice";

export const Chart = () => {
    const ids = useSelector(selectChartFocus);
    const lines = useSelector(selectChartLines(ids));
    const labels = Object.values(lines).map((line, i)=> {
        return (<LineWithDescp key={i} data={line.data}> {line.descp} </LineWithDescp>)
    });
    return (
        <div className={styles.container}>
            <div className={styles.list}>
                { labels }
            </div>
        </div>
    );
};
