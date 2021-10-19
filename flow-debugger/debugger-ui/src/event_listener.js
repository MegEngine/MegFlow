import { useState } from "react";
import { pin, unpin } from "./page/editor/EditorSlice";
import { update } from "./page/topology/TopologySlice";
import { update as update_err } from "./page/text/TextSlice";
import { changeMainPage, restoreDebuggerPage, changeStage } from "./app/actor";
import { ERROR, TOPOLOGY, EDITOR } from "./types";
import { parse } from "./parser";
import toml from "toml";

export default (dispatch, notify) => {
    const [parsed, setParsed] = useState({});
    return [parsed, {
        _onerror: (e) => {
            dispatch(update_err(`WEBSOCKET ERROR: ${e}`));
            dispatch(restoreDebuggerPage(ERROR));
            notify('connect')
            notify('disconnect')
        },
        _onclose: (e) => {
            dispatch(changeStage(""));
            dispatch(unpin());
            dispatch(changeMainPage(EDITOR));
            notify('connect')
            notify('disconnect')
        },
        initialized: (msg) => {
            try {
                dispatch(pin(msg.graph));
                const [for_viz, tmp] = parse(toml.parse(msg.graph));
                setParsed(tmp);
                dispatch(update(for_viz));
                dispatch(changeMainPage(TOPOLOGY));
            } catch (e) {
                if (e.line) {
                    dispatch(
                        update_err(
                            `ERROR: ${e.message} at lint ${e.line}, column ${e.column}`
                        )
                    );
                } else {
                    dispatch(update_err(`ERROR: ${e.message}`));
                }
                dispatch(restoreDebuggerPage(ERROR));
            }
            notify('connect')
        },
        terminated: (msg) => {
            notify('disconnect')
        },
    }];
};
