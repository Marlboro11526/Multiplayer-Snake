import { createSlice } from "@reduxjs/toolkit";

export const gameSlice = createSlice({
	name: "game_state",
	initialState: {
		players: [],
		arena_width: null,
		arena_height: null,
		food: [],
	},
	reducers: {
		setPlayers: (state, action) => {
			// console.debug("Set players", action);
			return {
				...state,
				players: action.payload,
			};
		},
		setArenaWidth: (state, action) => {
			// console.debug(action);
			// console.debug("action payload: ", action.payload);
			return {
				...state,
				arena_width: action.payload,
			};
		},
		setArenaHeight: (state, action) => {
			return {
				...state,
				arena_height: action.payload,
			};
		},
		setFood: (state, action) => {
			return {
				...state,
				food: action.payload,
			};
		},
	},
});

export const { setPlayers, setArenaWidth, setArenaHeight, setFood } =
	gameSlice.actions;

export default gameSlice.reducer;
