import { store } from "./store";
import { browserWidthChanged } from "./actor";

const z = (evt) => store.dispatch(browserWidthChanged(evt.matches));
const maxWidthMediaQuery = window.matchMedia("(max-width: 1600px)");
z(maxWidthMediaQuery);
maxWidthMediaQuery.addListener(z);

export * from "./actor";
export * from "./store";
