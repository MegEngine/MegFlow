import React, { useMemo } from "react";
import styles from "./LineWithDescp.module.css";
import { Chart } from "react-charts";

export const LineWithDescp = ({ data, children, ...props }) => {
    const primaryAxis = useMemo(
        () => ({
            getValue: (datum) => datum.primary,
        }),
        []
    );

    const secondaryAxes = useMemo(
        () => [
            {
                getValue: (datum) => datum.secondary,
            },
        ],
        []
    );
    let convert_data = [];
    for (const k in data) {
        convert_data.push({
            label: k,
            data: data[k].map((v, i)=>{
                return {
                    primary: i,
                    secondary: v,
                }
            }),
        })
    }

    return (
        <div className={styles.container} {...props}>
            <div className={styles.descp}> {children} </div>
            <div className={styles.line}>
                <Chart options={{ data: convert_data, primaryAxis, secondaryAxes }} />
            </div>
        </div>
    );
};
