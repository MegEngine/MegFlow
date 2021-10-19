import { createSlice } from "@reduxjs/toolkit";

export const chartSlice = createSlice({
    name: "chart",
    initialState: {
        lines: {},
        focus: [],
    },
    reducers: {
        append: (state, elem) => {
            if (!(elem.payload.id in state.lines)) {
                state.lines[elem.payload.id] = { descp: elem.payload.descp, start: Date.now(), data: {} };
            }
            let dst = state.lines[elem.payload.id].data;
            let start = state.lines[elem.payload.id].start;
            let src = elem.payload.data;

            let now = Date.now();
            let dur = now - start;

            for (const k in src) {
                if (!(k in dst)) {
                    dst[k] = [];
                }
                const len = dst[k].length;
                if (dur > 1000 || len === 0) {
                    dst[k].push(src[k]);
                } else {
                    dst[k][len-1] = Math.max(dst[k][len-1], src[k]);
                }
                if (len >= 60) { // TODO: remove the hardcode
                    dst[k].shift();     
                }
            }
            if (dur > 1000) { // TODO: remove the hardcode
                state.lines[elem.payload.id].start = Date.now();
            }
        },
        reset: (state) => {
            state.lines = {};
            state.focus = [];
        },
        focus: (state, ids) => {
            state.focus = ids.payload;
        }
    },
});

export const { append, reset, focus } = chartSlice.actions;

export const selectChartLines = (ids) => (state) => {
    let lines = {};
    for (const id of ids) {
        if (id in state.chart.lines) {
            lines[id] = state.chart.lines[id];
        }
    }
    return lines;
};

export const selectChartFocus = (state) => state.chart.focus;

export default chartSlice.reducer;
