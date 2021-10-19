import React, { useEffect, useRef } from "react";
import { useSelector } from "react-redux";
import Split from "split-grid";
import styles from "./ResizableArea.module.css";
import { Orientation } from "../../types";

const orientationSelect = (state) =>
    state.isSmall ? Orientation.Horizontal : Orientation.Vertical;

export function ResizableArea(props) {
    const orientation = useSelector(orientationSelect);

    const grid = useRef(null);
    const dragHandle = useRef(null);

    useEffect(() => {
        grid.current.style["grid-template-columns"] = null;
        grid.current.style["grid-template-rows"] = null;
    }, [orientation, props.focus]);

    useEffect(() => {
        const split = Split({
            minSize: 150,
            [TRACK_OPTION_NAME[orientation]]: [
                {
                    track: 1,
                    element: dragHandle.current,
                },
            ],
        });
        return () => split.destroy();
    }, [orientation, props.focus]);

    const gridStyle = props.focus
        ? GRID_STYLE[orientation]
        : HIDDEN_GRID_STYLE[orientation];
    const [handleOuterStyle, handleInnerStyle] = HANDLE_STYLES[orientation];

    return (
        <div ref={grid} className={gridStyle}>
            {props.children[0]}
            {props.focus && (
                <div ref={dragHandle} className={handleOuterStyle}>
                    <span className={handleInnerStyle}>⣿</span>
                </div>
            )}
            {props.children[1]}
        </div>
    );
}

const TRACK_OPTION_NAME = {
    [Orientation.Horizontal]: "rowGutters",
    [Orientation.Vertical]: "columnGutters",
};

const GRID_STYLE = {
    [Orientation.Horizontal]: styles.resizeableAreaRow,
    [Orientation.Vertical]: styles.resizeableAreaColumn,
};

const HIDDEN_GRID_STYLE = {
    [Orientation.Horizontal]: styles.resizeableHiddenAreaRow,
    [Orientation.Vertical]: styles.resizeableHiddenAreaColumn,
};

const HANDLE_STYLES = {
    [Orientation.Horizontal]: [
        styles.splitRowsGutter,
        styles.splitRowsGutterHandle,
    ],
    [Orientation.Vertical]: [styles.splitColumnsGutter, ""],
};
