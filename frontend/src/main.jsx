import React from "react";
import ReactDOM from "react-dom/client";

import { Provider } from 'react-redux'
import store from "./store";

import Game from "./features/Game";

ReactDOM.createRoot(document.getElementById("root")).render(
  <Provider store={store}>
    <React.StrictMode>
      <Game />
    </React.StrictMode>
  </Provider>
);
