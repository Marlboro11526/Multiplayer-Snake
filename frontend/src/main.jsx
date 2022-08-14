import React from "react";
import ReactDOM from "react-dom/client";

import { Provider } from "react-redux";
import store from "./redux_logic/store";

import { BrowserRouter, Routes, Route } from "react-router-dom";

import Game from "./features/Game";

ReactDOM.createRoot(document.getElementById("root")).render(
	<Provider store={store}>
		<React.StrictMode>
			<BrowserRouter>
				<Routes>
					<Route path="/" element={<Game />} />
				</Routes>
			</BrowserRouter>
		</React.StrictMode>
	</Provider>
);
