import React, { useState } from "react";
import { useSelector, useDispatch } from "react-redux";
import styles from "./AppHeader.module.css";
import { Button } from "./component/button/Button";
import { Input } from "./component/input/Input";
import toml from "toml";
import { save, selectEditorChanged } from "./page/editor/EditorSlice";
import { update as update_err } from "./page/text/TextSlice";
import { parse, splitQpsNode } from "./parser";
import { update, setBlock, listen } from "./page/topology/TopologySlice";
import { changeMainPage, restoreDebuggerPage, storeDebuggerPage, changeStage } from "./app/actor";
import { TOPOLOGY, ERROR } from "./types";
import Popup from "reactjs-popup";
import { useWait } from "react-wait";
import "reactjs-popup/dist/index.css";
import { useWebsocket } from "./websocket";
import buildEventListener from "./event_listener";
import { append } from "./page/chart/ChartSlice";

const Spinner = () => {
    return (
        <img
            alt=""
            className={styles.spinner}
            src="https://i.pinimg.com/originals/3e/f0/e6/3ef0e69f3c889c1307330c36a501eb12.gif"
        />
    );
};

const SwitchButton = (is_on, off, on, onClick, disabled) => {
    return is_on ? (
        <Button red bold onClick={onClick} disabled={disabled}>
            {off}
        </Button>
    ) : (
        <Button onClick={onClick} disabled={disabled}>
            {on}
        </Button>
    );
};

export const AppHeader = () => {
    const dispatch = useDispatch();
    const [form, openForm] = useState(false);
    const [perf, setPerf] = useState(false);
    const [url, setUrl] = useState("");
    const [qpsSeqId, setQpsSeqId] = useState(-1);
    const stage = useSelector((state) => state.stage);
    const content = useSelector(selectEditorChanged);
    const { startWaiting, endWaiting, isWaiting, Wait } = useWait();
    const [parsed, onEvent] = buildEventListener(dispatch, (event) => {
        switch (event) {
            case "disconnect":
                dispatch(changeStage(""));
                setPerf(false);
                break
            case "connect":
                openForm(false);
                dispatch(changeStage("dbg-main"));
                break
            default:
                break
        }
        endWaiting(event);
    });
    const { send, connect, disconnect } = useWebsocket(onEvent);

    switch (stage) {
        case "dbg-main":
            return (
                <div className={styles.AppHeader}>
                    <div className={styles.ButtonSet}>
                        <Button
                            bold
                            disabled={isWaiting("disconnect")}
                            onClick={() => {
                                startWaiting("disconnect");
                                disconnect();
                            }}
                        >
                            <Wait on="disconnect" fallback={<Spinner />}>
                                disconnect
                            </Wait>
                        </Button>
                    </div>
                    <div className={styles.ButtonSet}>
                        {SwitchButton(
                            perf,
                            "Stop",
                            "Perf",
                            () => {
                                if (!perf) {
                                    setQpsSeqId(send(qpsSeqId, {
                                        feature: 'QPS',
                                        command: 'start',
                                        ratio: 1.0,
                                    }, (resp) => {
                                        let is_block = [];
                                        for (const node of resp.nodes) {
                                            let split = splitQpsNode(parsed, resp.graph, node);
                                            if (!split) {
                                                return;
                                            }
                                            if (split.is_block) {
                                                if (split.id) 
                                                    is_block.push(split.id);
                                                else
                                                    for (const id of split.ids)
                                                        is_block.push(id)
                                            }
                                            for (const port of split.ports) {
                                                dispatch(append(port));
                                            }
                                        }
                                        dispatch(setBlock(is_block));
                                    }))
                                    dispatch(listen(true));
                                } else {
                                    send(qpsSeqId, {
                                        feature: 'QPS',
                                        command: 'stop',
                                    });
                                    setQpsSeqId(-1);
                                    dispatch(listen(false));
                                    dispatch(restoreDebuggerPage(''));
                                    dispatch(storeDebuggerPage());
                                }
                                setPerf(!perf);
                            },
                            isWaiting("disconnect")
                        )}
                    </div>
                </div>
            );
        default:
            return (
                <div className={styles.AppHeader}>
                    <div className={styles.ButtonSet}>
                        <Button
                            bold
                            children="Launch"
                            onClick={() => {
                                openForm(true);
                                setUrl('');
                            }}
                        />
                        <Button
                            bold
                            children="Attach"
                            onClick={() => {
                                openForm(true);
                                setUrl('');
                            }}
                        />
                    </div>
                    <div className={styles.ButtonSet}>
                        <Button
                            children="Check"
                            onClick={() => {
                                try {
                                    const [for_viz] = parse(
                                        toml.parse(content)
                                    );
                                    dispatch(update(for_viz));
                                    dispatch(update_err(""));
                                    dispatch(changeMainPage(TOPOLOGY));
                                    dispatch(save());
                                } catch (e) {
                                    if (e.line) {
                                        dispatch(
                                            update_err(
                                                `ERROR: ${e.message} at lint ${e.line}, column ${e.column}`
                                            )
                                        );
                                    } else {
                                        dispatch(
                                            update_err(`ERROR: ${e.message}`)
                                        );
                                    }
                                    dispatch(restoreDebuggerPage(ERROR));
                                }
                            }}
                        />
                    </div>
                    <Popup
                        open={form}
                        repositionOnResize
                        closeOnDocumentClick={false}
                        contentStyle={{
                            width: "auto",
                            borderRadius: "8px 8px 8px 8px",
                        }}
                    >
                        <div className={styles.PopupContent}>
                            <Input
                                label="target"
                                onChange={(e) => setUrl(`ws://${e.target.value}/debugger`)}
                            />
                            <div className={styles.ButtonSet}>
                                <Button
                                    disabled={isWaiting("connect")}
                                    onClick={() => {
                                        startWaiting("connect");
                                        connect(url);
                                    }}
                                >
                                    <Wait on="connect" fallback={<Spinner />}>
                                        connect
                                    </Wait>
                                </Button>
                            </div>
                        </div>
                    </Popup>
                </div>
            );
    }
};
