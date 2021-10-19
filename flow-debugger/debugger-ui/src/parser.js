function list2map(list) {
    if (list) {
        return Object.fromEntries(list.map((e) => [e.name, e]));
    } else {
        return {};
    }
}

function isGraph(config, name) {
    return config.graphs.hasOwnProperty(name);
}

function isInput(graph, name) {
    return graph.inputs.hasOwnProperty(name);
}

function isOutput(graph, name) {
    return graph.outputs.hasOwnProperty(name);
}

function parseGraph(config, graph) {
    let nodes = {};
    let edges = [];
    let inputs = {};
    let inputs_tag = {};
    let outputs = {};
    let outputs_tag = {};

    let subgraphs = {};
    let dots = [];

    for (const name in graph.nodes) {
        let node = graph.nodes[name];
        if (isGraph(config, node.ty)) {
            subgraphs[name] = parseGraph(config, config.graphs[node.ty]);
            node.link = subgraphs[name];
        } else {
            nodes[name] = {
                id: config.counter,
                label: name,
                color: "#fff",
                title: graph.name,
                ports: new Set(),
            };
            node.link = nodes[name];
            config.counter += 1;
        }
    }

    for (const conn of graph.connections) {
        let related = { rx: [], tx: [], unknown: [] };
        for (const port of conn.ports) {
            let port_split = port.split(":");
            let node_name = port_split[0];
            let node_port = port_split[1];
            if (nodes.hasOwnProperty(node_name)) {
                nodes[node_name].ports.add(node_port);
                related.unknown.push(nodes[node_name].id);
            } else if (subgraphs.hasOwnProperty(node_name)) {
                const subgraph = subgraphs[node_name];
                if (isInput(subgraph, node_port)) {
                    for (const id of subgraph.inputs[node_port]) {
                        related.rx.push(id);
                    }
                } else if (isOutput(subgraph, node_port)) {
                    for (const id of subgraph.outputs[node_port]) {
                        related.tx.push(id);
                    }
                } else {
                    throw new Error(
                        `port[${node_port}] is not found in graph[${graph.nodes[node_name].ty}]`
                    );
                }
            } else if (config.g_nodes.hasOwnProperty(node_name)) {
                config.g_nodes[node_name].ports.add(node_port);
                related.unknown.push(config.g_nodes[node_name].id);
            } else if (config.g_graphs.hasOwnProperty(node_name)) {
                const subgraph = config.g_graphs[node_name];
                if (isInput(subgraph, node_port)) {
                    for (const id of subgraph.inputs[node_port]) {
                        related.rx.push(id);
                    }
                } else if (isOutput(subgraph, node_port)) {
                    for (const id of subgraph.outputs[node_port]) {
                        related.tx.push(id);
                    }
                } else {
                    throw new Error(
                        `port[${node_port}] is not found in graph[${graph.nodes[node_name].ty}]`
                    );
                }
            } else {
                throw new Error(`node[${node_name}] is not found in config`);
            }
        }
        if (
            related.rx.length + related.tx.length + related.unknown.length >
            2
        ) {
            let dot = config.counter;
            dots.push(dot);
            config.counter += 1;
            for (const rx of related.rx) {
                edges.push({ from: dot, to: rx, arrows: "to" });
            }
            for (const tx of related.tx) {
                edges.push({ from: tx, to: dot, arrows: "to" });
            }
            for (const unknown of related.unknown) {
                edges.push({ from: unknown, to: dot, arrows: "" });
            }
        } else {
            let from = -1,
                to = -1,
                offset = 0;
            if (related.tx.length === 1) {
                from = related.tx[0];
            }
            if (related.rx.length === 1) {
                to = related.rx[0];
            }
            if (from === -1) {
                from = related.unknown[offset];
                offset += 1;
            }
            if (to === -1) {
                to = related.unknown[offset];
                offset += 1;
            }
            edges.push({ from, to, arrows: offset === 2 ? "" : "to" });
        }
    }

    for (const name in graph.inputs) {
        const input = graph.inputs[name];
        let ids = [];
        let tags = [];
        for (const port of input.ports) {
            let port_split = port.split(":");
            let node_name = port_split[0];
            let port_name = port_split[1];
            if (nodes.hasOwnProperty(node_name)) {
                nodes[node_name].ports.add(port_name);
                ids.push(nodes[node_name].id);
                tags.push(port_name)
            } else if (subgraphs.hasOwnProperty(node_name)) {
                for (const id of subgraphs[node_name].inputs[port_name]) {
                    ids.push(id);
                }
                for (const tag of subgraphs[node_name].inputs_tag[port_name]) {
                    tags.push(tag);
                }
            } else if (config.g_nodes.hasOwnProperty(node_name)) {
                config.g_nodes[node_name].ports.add(port_name);
                ids.push(config.g_nodes[node_name].id);
                tags.push(port_name);
            } else if (config.g_graphs.hasOwnProperty(node_name)) {
                const subgraph = config.g_graphs[node_name];
                for (const id of subgraph[node_name].inputs[port_name]) {
                    ids.push(id);
                }
                for (const tag of subgraphs[node_name].inputs_tag[port_name]) {
                    tags.push(tag);
                }
            } else {
                throw new Error(`node[${node_name}] is not found in config`);
            }
        }
        inputs[name] = ids;
        inputs_tag[name] = tags;
    }

    for (const name in graph.outputs) {
        const output = graph.outputs[name];
        let ids = [];
        let tags = [];
        for (const port of output.ports) {
            let port_split = port.split(":");
            let node_name = port_split[0];
            let port_name = port_split[1];
            if (nodes.hasOwnProperty(node_name)) {
                nodes[node_name].ports.add(port_name);
                ids.push(nodes[node_name].id);
                tags.push(port_name);
            } else if (subgraphs.hasOwnProperty(node_name)) {
                for (const id of subgraphs[node_name].outputs[port_name]) {
                    ids.push(id);
                }
                for (const tag of subgraphs[node_name].outputs_tag[port_name]) {
                    tags.push(tag);
                }
            } else if (config.g_nodes.hasOwnProperty(node_name)) {
                config.g_nodes[node_name].ports.add(port_name);
                ids.push(config.g_nodes[node_name].id);
                tags.push(port_name);
            } else if (config.g_graphs.hasOwnProperty(node_name)) {
                const subgraph = config.g_graphs[node_name];
                for (const id of subgraph[node_name].outptus) {
                    ids.push(id);
                }
                for (const tag of subgraphs[node_name].outputs_tag[port_name]) {
                    tags.push(tag);
                }
            } else {
                throw new Error(`node[${node_name}] is not found in config`);
            }
        }
        outputs[name] = ids;
        outputs_tag = tags;
    }

    return {
        name: graph.name,
        nodes: Object.values(nodes)
            .concat(
                dots.map((id) => {
                    return {
                        id,
                        label: null,
                        color: "#000",
                        shape: "dot",
                        size: 1,
                    };
                })
            )
            .concat(
                Object.values(subgraphs).reduce(
                    (cal, g) => cal.concat(g.nodes),
                    []
                )
            ),
        edges: edges.concat(
            Object.values(subgraphs).reduce((cal, g) => cal.concat(g.edges), [])
        ),
        inputs,
        inputs_tag,
        outputs,
        outputs_tag,
    };
}

function translateGraph(graph) {
    return {
        name: graph.name,
        nodes: list2map(graph.nodes) || {},
        inputs: list2map(graph.inputs) || {},
        outputs: list2map(graph.outputs) || {},
        connections: graph.connections || [],
    };
}

function translateConfig(config) {
    return {
        main: config.main,
        nodes: list2map(config.nodes) || {},
        graphs:
            list2map(config.graphs.map((graph) => translateGraph(graph))) || {},
        counter: 1,
    };
}

export const parse = (config) => {
    let config_translated = translateConfig(config);
    let main = config.main;
    config_translated.g_nodes = {};
    config_translated.g_graphs = {};

    for (const name in config_translated.nodes) {
        let node = config_translated.nodes[name];
        if (isGraph(config_translated, node.ty)) {
            config_translated.g_graphs[name] = parseGraph(
                config_translated,
                config_translated.graphs[node.ty]
            );
            node.link = config_translated.g_graphs[name];
        } else {
            config_translated.g_nodes[name] = {
                id: config_translated.counter,
                label: name,
                color: "#fff",
                title: "__GLOBAL__",
                ports: new Set(),
            };
            node.link = config_translated.g_nodes[name];
            config_translated.counter += 1;
        }
    }

    let parsed = parseGraph(config_translated, config_translated.graphs[main]);

    for (const name in parsed.inputs) {
        let id = config_translated.counter;
        parsed.nodes.push({
            id,
            label: name,
            color: "#000",
            font: { color: "#fff" },
        });
        config_translated.counter += 1;
        for (const to of parsed.inputs[name]) {
            parsed.edges.push({ from: id, to, arrows: "to" });
        }
    }

    for (const name in parsed.outputs) {
        let id = config_translated.counter;
        parsed.nodes.push({
            id,
            label: name,
            color: "#000",
            font: { color: "#fff" },
        });
        config_translated.counter += 1;
        for (const from of parsed.outputs[name]) {
            parsed.edges.push({ from, to: id, arrows: "to" });
        }
    }
    return [
        {
            nodes: parsed.nodes.concat(
                Object.values(config_translated.g_nodes)
            ).map(node=> {
                if (node.ports) {
                    node.ports = [...node.ports];
                }
                return node;
            }),
            edges: parsed.edges,
        },
        config_translated,
    ];
};

function findNodeById(config, graph_name, id) {
    for (const node of Object.values(config.graphs[graph_name].nodes)) {
        if ((id in node.link) && node.link.id === id) {
            return node;
        }
    }
    for (const node of Object.values(config.nodes)) {
        if ((id in node.link) && node.link.id === id) {
            return node;
        }
    }
    for (const graph of Object.values(config.graphs)) {
        if (graph.name === graph_name) continue;
        for (const node of Object.values(graph.nodes)) {
            if ((id in node.link) && node.link.id === id) {
                return node;
            }   
        }
    }
    return null;
}

export const splitQpsNode = (config, graph_name, node) => {
    let splitNode = (cfg) => {
        if (!cfg) return undefined;
        let id = cfg.link.id;
        let ports = Object.entries(node.qps).map(pair=>{
            return { id: `${id}#${pair[0]}`, descp: `${node.name}:${pair[0]}`, data: { size: pair[1][0], qps: pair[1][1] } };
        });
        return { id, is_block: node.is_block, ports };
    };
    let splitGraph = (cfg) => {
        if (!cfg) return undefined;
        let ids = [];
        let inputs = (pair) => cfg.link.inputs[pair[0]].map((id, i)=> {
            ids.push(id);
            let node = findNodeById(config, cfg.ty, id); 
            let tag = cfg.link.inputs_tag[pair[0]][i];
            return { id: `${id}#${tag}`, descp: `${node.name}:${tag}`, data: { size: pair[1][0], qps: pair[1][1] } }
        });
        let outputs = (pair) => cfg.link.outputs[pair[0]].map((id, i)=> {
            let node = findNodeById(config, cfg.ty, id);
            let tag = cfg.link.outputs_tag[pair[0]][i];
            return { id: `${id}#${tag}`, descp: `${node.name}:${tag}`, data: { size: pair[1][0], qps: pair[1][1] } }
        });
        let ports = Object.entries(node.qps).map(pair=>{
            return inputs(pair).concat(outputs(pair))  
        }).flat();
        return { ids, is_block: node.is_block, ports };
    };
    if (node.name in config.g_nodes) {
        let cfg = config.nodes[node.name];
        return splitNode(cfg);
    }
    if (node.name in config.g_graphs) {
        let cfg = config.nodes[node.name];
        return splitGraph(cfg);
    }
    let cfg = config.graphs[graph_name].nodes[node.name];
    if (!cfg) return undefined;
    if (isGraph(config, cfg.ty)) {
        return splitGraph(cfg);
    } else {
        return splitNode(cfg);
    }
}
