import React from "react";
import { useSelector } from "react-redux";
import { Empty } from "./empty/Empty";
import { Text } from "./text/Text";
import { Topology } from "./topology/Topology";
import { Editor } from "./editor/Editor";
import { Chart } from "./chart/Chart";

export const MainSwitcher = () => {
    const page = useSelector((state) => state.main.page);
    switch (page) {
        case "topology":
            return <Topology />;
        case "editor":
            return <Editor />;
        default:
            return <Editor />;
    }
};

export const DebuggerSwitcher = () => {
    const page = useSelector((state) => state.dbg.page);
    
    switch (page) {
        case "error":
            return <Text />;
        case "chart":
            return <Chart />;
        default:
            return <Empty />;
    }
};
