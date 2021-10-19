import React from "react";
import { useSelector, useDispatch } from "react-redux";
import { selectDescp, selectListen } from "./TopologySlice";
import Graph from "./VisGraph";
import { restoreDebuggerPage, storeDebuggerPage } from "../../app/actor";
import { CHART } from "../../types";
import { focus } from "../chart/ChartSlice";

const options = {
    autoResize: true,
    width: "100%",
    height: "100%",
    layout: {
        randomSeed: 0,
        hierarchical: false,
    },
    edges: {
        color: "#ffffff",
    },
    physics: {
        enabled: false,
    },
};

export const Topology = () => {
    let dispatch = useDispatch();
    let descp = useSelector(selectDescp);
    let listen = useSelector(selectListen);
    let edges = descp.edges.map((edge) => {
        return { ...edge };
    }); // workaround for graph-viz
    let events = listen ? {
        click: (e) => {
            if (e.nodes.length === 1) {
                let id = e.nodes[0];
                let ports = descp.nodes.find(node=>node.id === id).ports;
                let ids = ports.map(port=>`${id}#${port}`);
                dispatch(focus(ids));
                dispatch(restoreDebuggerPage(CHART));
            }
        },
        deselectNode: (e) => {
            if (e.nodes.length === 0) {
                dispatch(storeDebuggerPage())
            }
        },
    } : {};
    return (
        <Graph
            graph={{ nodes: descp.nodes, edges }}
            style={{ height: "100%" }}
            events={events}
            options={options}
        />
    );
};
