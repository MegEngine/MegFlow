import React from "react";
import { useSelector, useDispatch } from "react-redux";
import "./App.css";
import { ResizableArea } from "./component/resizable_area/ResizableArea";
import { TabList } from "./component/tab/Tab";
import { MainSwitcher, DebuggerSwitcher } from "./page";
import { changeMainPage, storeDebuggerPage, restoreDebuggerPage } from "./app";
import { AppHeader } from "./AppHeader";
import { save } from "./page/editor/EditorSlice";
import { MainTabs, EDITOR, TOPOLOGY } from "./types";

function App() {
    const dispatch = useDispatch();
    const main = useSelector((state) => state.main);
    const dbg = useSelector((state) => state.dbg);

    return (
        <div className="App">
            <AppHeader />
            <ResizableArea focus={dbg.focus >= 0}>
                <div className="BorderArea">
                    <TabList
                        focus={main.focus}
                        tabs={[
                            {
                                label: MainTabs[EDITOR],
                                onClick: () => dispatch(changeMainPage(EDITOR)),
                            },
                            {
                                label: MainTabs[TOPOLOGY],
                                onClick: () =>
                                    dispatch(changeMainPage(TOPOLOGY)) &
                                    dispatch(save()),
                            },
                        ]}
                    />
                    <div className="body">
                        <MainSwitcher />
                    </div>
                </div>
                <div className={dbg.focus >= 0 ? "BorderArea" : "Area"}>
                    <TabList
                        focus={dbg.focus}
                        tabs={[
                            {
                                label: "debugger",
                                onClick: () =>
                                    dispatch(restoreDebuggerPage(-1)),
                            },
                        ]}
                        children={
                            dbg.focus >= 0 && (
                                <button
                                    className="tabClose"
                                    onClick={() =>
                                        dispatch(storeDebuggerPage())
                                    }
                                >
                                    Close
                                </button>
                            )
                        }
                    />
                    {dbg.focus >= 0 && (
                        <div className="body">
                            <DebuggerSwitcher />
                        </div>
                    )}
                </div>
            </ResizableArea>
        </div>
    );
}

export default App;
