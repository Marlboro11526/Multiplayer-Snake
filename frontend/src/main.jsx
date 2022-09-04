import React from "react";
import ReactDOM from "react-dom/client";

import { Provider } from "react-redux";
import store from "./redux_logic/store";

import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { Arena } from "./features/Arena";
import { Landing } from "./features/Landing";

ReactDOM.createRoot(document.getElementById("root")).render(
	<Provider store={store}>
		<React.StrictMode>
			<BrowserRouter>
				<Routes>
					<Route path="arena" element={<Arena />} />
					<Route path="landing" element={<Landing />} />
					<Route path="*" element={<Navigate to="landing" />} />
				</Routes>
			</BrowserRouter>
		</React.StrictMode>
	</Provider>
);
