import { createAction } from "@reduxjs/toolkit";

export const BrowserWidthChanged = "BROWSER_WIDTH_CHANGED";
export const browserWidthChanged = createAction(
    BrowserWidthChanged,
    (isSmall) => {
        return {
            payload: {
                isSmall,
            },
        };
    }
);

export const StoreDebuggerPage = "STORE_DEBUGGER_PAGE";
export const storeDebuggerPage = createAction(StoreDebuggerPage, () => {
    return {
        payload: {},
    };
});

export const RestoreDebuggerPage = "RESTORE_DEBUGGER_PAGE";
export const restoreDebuggerPage = createAction(
    RestoreDebuggerPage,
    (focus) => {
        return {
            payload: {
                focus,
            },
        };
    }
);

export const ChangeMainPage = "CHANGE_MAIN_PAGE";
export const changeMainPage = createAction(ChangeMainPage, (focus) => {
    return {
        payload: {
            focus,
        },
    };
});

export const ChangeStage = "CHANGE_STAGE";
export const changeStage = createAction(ChangeStage, (stage) => {
    return {
        payload: {
            stage,
        },
    };
});
