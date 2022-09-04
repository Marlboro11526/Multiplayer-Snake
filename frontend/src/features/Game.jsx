import React from "react";
import { useEffect } from "react";

import { Arena } from "./Arena";
import { Landing } from "./Landing";
import { Gateway } from "./Gateway";

import { Routes, Route, useNavigate } from "react-router-dom";
import { useSelector } from "react-redux";
import { playerNameSelector } from "../redux_logic/selectors";

const gateway = new Gateway();

function Game() {
	const player_name = useSelector(playerNameSelector);
	const navigate = useNavigate();

	useEffect(() => {
		gateway.start();

		if (player_name != null) {
			navigate("arena");
		} else {
			navigate("landing");
		}
	}, [player_name]);

	return (
		<Routes>
			<Route path="arena" element={<Arena />} />
			<Route path="landing" element={<Landing />} />
		</Routes>
	);
}

export default Game;
