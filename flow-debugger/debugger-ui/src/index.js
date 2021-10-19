import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import { store } from "./app";
import App from "./App";
import { Provider } from "react-redux";
import * as serviceWorker from "./serviceWorker";
import { Waiter } from "react-wait";

ReactDOM.render(
    <React.StrictMode>
        <Waiter>
            <Provider store={store}>
                <App />
            </Provider>
        </Waiter>
    </React.StrictMode>,
    document.getElementById("root")
);

serviceWorker.register();
