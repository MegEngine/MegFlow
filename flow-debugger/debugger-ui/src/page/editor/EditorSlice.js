import { createSlice } from "@reduxjs/toolkit";

export const editorSlice = createSlice({
    name: "editor",
    initialState: {
        init: "",
        changed: "",
        edit: true,
    },
    reducers: {
        save: (state) => {
            state.init = state.changed;
        },
        update: (state, changed) => {
            state.changed = changed.payload;
        },
        pin: (state, overwrite) => {
            state.init = overwrite.payload;
            state.changed = overwrite.payload;
            state.edit = false;
        },
        unpin: (state) => {
            state.edit = true;
        },
    },
});

export const { save, update, pin, unpin } = editorSlice.actions;

export const selectEditorInit = (state) => state.editor.init;
export const selectEditorChanged = (state) => state.editor.changed;
export const selectEdit = (state) => state.editor.edit;

export default editorSlice.reducer;
