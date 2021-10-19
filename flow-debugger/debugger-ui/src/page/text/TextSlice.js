import { createSlice } from "@reduxjs/toolkit";

export const textSlice = createSlice({
    name: "text",
    initialState: {
        message: "",
    },
    reducers: {
        update: (state, changed) => {
            state.message = changed.payload;
        },
    },
});

export const { update } = textSlice.actions;

export const selectTextMessage = (state) => state.text.message;

export default textSlice.reducer;
