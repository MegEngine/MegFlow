import React, {
    forwardRef,
    useEffect,
    useImperativeHandle,
    useRef,
    useState,
} from "react";
import { DataSet } from "vis-data";
import { Network } from "vis-network";
import {
    differenceWith,
    intersectionWith,
    isEqual,
    defaultsDeep,
    cloneDeep,
} from "lodash";

var __rest =
    (this && this.__rest) ||
    function (s, e) {
        var t = {};
        for (var p in s)
            if (Object.prototype.hasOwnProperty.call(s, p) && e.indexOf(p) < 0)
                t[p] = s[p];
        if (s != null && typeof Object.getOwnPropertySymbols === "function")
            for (
                var i = 0, pp = Object.getOwnPropertySymbols(s);
                i < pp.length;
                i++
            ) {
                if (
                    e.indexOf(pp[i]) < 0 &&
                    Object.prototype.propertyIsEnumerable.call(s, pp[i])
                )
                    t[pp[i]] = s[pp[i]];
            }
        return t;
    };

/**
 * Keeps the value the same permanently.
 * Useful over refs especially in instances where the function creation variant is used
 */
function useSealedState(value) {
    const [state] = useState(value);
    return state;
}
/**
 * https://github.com/crubier/react-graph-vis/commit/68bf2e27b2046d6c0bb8b334c2cf974d23443264
 */
const diff = (current, next, field = "id") => {
    const nextIds = new Set(next.map((item) => item[field]));
    const removed = current.filter((item) => !nextIds.has(item[field]));
    const unchanged = intersectionWith(next, current, isEqual);
    const updated = differenceWith(
        intersectionWith(next, current, (a, b) => a[field] === b[field]),
        unchanged,
        isEqual
    );
    const added = differenceWith(
        differenceWith(next, current, isEqual),
        updated,
        isEqual
    );
    return {
        removed,
        unchanged,
        updated,
        added,
    };
};
const defaultOptions = {
    physics: {
        stabilization: false,
    },
    autoResize: false,
    edges: {
        smooth: false,
        color: "#000000",
        width: 0.5,
        arrows: {
            to: {
                enabled: true,
                scaleFactor: 0.5,
            },
        },
    },
};
/**
 * Conversion of https://github.com/crubier/react-graph-vis/blob/master/src/index.js to a function component
 */
const VisGraph = forwardRef((_a, ref) => {
    var { graph, events, options: propOptions } = _a,
        props = __rest(_a, ["graph", "events", "options"]);
    const container = useRef(null);
    const edges = useSealedState(() => new DataSet(graph.edges));
    const nodes = useSealedState(() => new DataSet(graph.nodes));
    const initialOptions = useSealedState(propOptions);
    const prevNodes = useRef(graph.nodes);
    const prevEdges = useRef(graph.edges);
    useEffect(() => {
        if (isEqual(graph.nodes, prevNodes.current)) {
            return; // No change!
        }
        const { added, removed, updated } = diff(
            prevNodes.current,
            graph.nodes
        );
        nodes.remove(removed);
        nodes.add(added);
        nodes.update(updated);
        prevNodes.current = graph.nodes;
    }, [graph.nodes, nodes]);
    useEffect(() => {
        if (isEqual(graph.edges, prevEdges.current)) {
            return; // No change!
        }
        const { added, removed, updated } = diff(
            prevEdges.current,
            graph.edges
        );
        edges.remove(removed);
        edges.add(added);
        edges.update(updated);
        prevEdges.current = graph.edges;
    }, [graph.edges, edges]);
    const [network, setNetwork] = useState();
    useImperativeHandle(ref, () => network, [network]);
    useEffect(() => {
        if (!network || !events) {
            return () => {};
        }
        // Add user provied events to network
        for (const [eventName, callback] of Object.entries(events)) {
            if (callback) {
                network.on(eventName, callback);
            }
        }
        return () => {
            for (const [eventName, callback] of Object.entries(events)) {
                if (callback) {
                    network.off(eventName, callback);
                }
            }
        };
    }, [events, network]);
    useEffect(() => {
        if (!network || !propOptions) {
            return;
        }
        try {
            network.setOptions(propOptions);
        } catch (error) {
            // Throws when it hot reloads... Yay
            if (process.env.NODE_ENV !== "development") {
                // Still throw it in prod where there's no hot reload
                throw error;
            }
        }
    }, [network, propOptions]);
    useEffect(() => {
        // Creating the network has to be done in a useEffect because it needs access to a ref
        // merge user provied options with our default ones
        // defaultsDeep mutates the host object
        const mergedOptions = defaultsDeep(
            cloneDeep(initialOptions),
            defaultOptions
        );
        const newNetwork = new Network(
            container.current,
            { edges, nodes },
            mergedOptions
        );
        setNetwork(newNetwork);
        return () => {
            // Cleanup the network on component unmount
            newNetwork.destroy();
        };
    }, [edges, initialOptions, nodes]);
    return React.createElement("div", Object.assign({ ref: container }, props));
});
export default VisGraph;
