import { configureStore } from "@reduxjs/toolkit";
import * as actor from "./actor";
import editor from "../page/editor/EditorSlice";
import topology from "../page/topology/TopologySlice";
import text from "../page/text/TextSlice";
import chart from "../page/chart/ChartSlice";
import { MainTabs, DebuggerTabs } from "../types";

export const store = configureStore({
    reducer: {
        editor,
        topology,
        text,
        chart,
        isSmall: (state = false, action) => {
            switch (action.type) {
                case actor.BrowserWidthChanged:
                    return action.payload.isSmall;
                default:
                    return state;
            }
        },
        main: (state = { focus: 0, page: "" }, action) => {
            switch (action.type) {
                case actor.ChangeMainPage:
                    return {
                        focus: action.payload.focus,
                        page: MainTabs[action.payload.focus],
                    };
                default:
                    return state;
            }
        },
        dbg: (state = { focus: -1, page: "" }, action) => {
            switch (action.type) {
                case actor.StoreDebuggerPage:
                    return { focus: -1, page: state.page };
                case actor.RestoreDebuggerPage:
                    return {
                        focus: 0,
                        page:
                            action.payload.focus >= 0
                                ? DebuggerTabs[action.payload.focus]
                                : state.page,
                    };
                default:
                    return state;
            }
        },
        stage: (state = "", action) => {
            switch (action.type) {
                case actor.ChangeStage:
                    return action.payload.stage;
                default:
                    return state;
            }
        },
    },
});
