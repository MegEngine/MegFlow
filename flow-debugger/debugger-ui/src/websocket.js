import { useState, useRef, useEffect } from "react";

export const useWebsocket = (onEvent) => {
    const ws = useRef(null);
    const [ seq, setSeq ] = useState(0);
    const [ registry, setRegistry ] = useState({});

    const createWebSocket = (url) => {
        try {
            ws.current = new WebSocket(url);
            ws.current.onclose = (e) => {
                onEvent._onclose && onEvent._onclose(e);
                ws.current = null;
            };
            ws.current.onerror = (e) => {
                onEvent._onerror && onEvent._onerror(`there was an error with your websocket`);
            };
            ws.current.onmessage = ({ data }) => {
                const parsed = JSON.parse(data);
                if (parsed.ty && parsed.ty === "event") {
                    if (parsed.event === "stop") {
                        let rest = registry;
                        delete rest[parsed.seq_id];
                        setRegistry(rest);
                    } else {
                        onEvent[parsed.event](parsed);
                    }
                } else if (parsed.ty && parsed.ty === "response") {
                    if (!(parsed.seq_id in registry))
                        return;
                    for (const callback of registry[parsed.seq_id]) {
                        callback(parsed);
                    }
                }
            };
        } catch (e) {
            onEvent._onerror && onEvent._onerror(`connect ${url} fault`);
            ws.current = null;
        }
    };

    const connect = (url) => {
        if (!ws.current) {
            createWebSocket(url);
        }
    };

    const disconnect = () => {
        ws.current && ws.current.close();
    };

    const send = (seq_id, msg, callback=undefined) => {
        msg.ty = 'request';
        if (seq_id >= 0)
            msg.seq_id = seq_id;
        else {
            msg.seq_id = seq;
            setSeq(seq + 1);
        }
        let tmp = registry;
        if (!(msg.seq_id in tmp)) {
            tmp[msg.seq_id] = []
        }
        if (callback) {
            tmp[msg.seq_id].push(callback)
        }
        setRegistry(tmp);
        ws.current && ws.current.send(JSON.stringify(msg));
        return msg.seq_id;
    };

    useEffect(() => {
        return () => {
            ws.current && ws.current.close();
        };
    }, [ws]);

    return {
        send,
        connect,
        disconnect,
    };
};
