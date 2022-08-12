import React from "react";

import { useSelector } from 'react-redux'
import { Arena } from "./Arena";
import { Landing } from './Landing'

function Game() {
	console.debug("GAME");
	const player_name = useSelector((state) => state.userState.name);
	if(player_name) {
		return <Arena />
	} else {
		return <Landing />
	}
}

export default Game;
