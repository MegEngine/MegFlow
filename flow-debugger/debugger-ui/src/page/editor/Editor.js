import React, { useMemo } from "react";
import { useDispatch, useSelector } from "react-redux";
import CodeMirror from "rodemirror";
import { basicSetup } from "@codemirror/basic-setup";
import { oneDark } from "@codemirror/theme-one-dark";
import { StreamLanguage } from "@codemirror/stream-parser";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { update, selectEditorInit, selectEdit } from "./EditorSlice";

export const Editor = () => {
    const dispatch = useDispatch();
    const init = useSelector(selectEditorInit);
    const edit = useSelector(selectEdit);
    const extensions = useMemo(
        () => [basicSetup, oneDark, StreamLanguage.define(toml)],
        []
    );
    return (
        <CodeMirror
            extensions={extensions}
            value={init}
            onUpdate={(v) => {
                if (v.docChanged && edit) {
                    dispatch(update(v.state.doc.toString()));
                }
            }}
        />
    );
};
