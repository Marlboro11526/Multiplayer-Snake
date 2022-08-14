import React from "react";
import { useEffect } from "react";

import { Arena } from "./Arena";
import { Landing } from './Landing'
import { Gateway } from './Gateway';

import { Routes, Route } from "react-router-dom";


function Game() {
	
	useEffect(() => {
		let gateway = new Gateway();
		gateway.start();
	});

	return (
		<Routes>
			<Route path='arena' element={<Arena />}/>
			<Route path='landing' element={<Landing />}/>
		</Routes>
	)
}

export default Game;
