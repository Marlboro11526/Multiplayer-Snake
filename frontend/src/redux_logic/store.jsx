import { configureStore } from "@reduxjs/toolkit";
import gameStateReducer from "./slices/gameStateSlice";
import userStateReducer from "./slices/userSlice";

export default configureStore({
	reducer: {
		gameState: gameStateReducer,
		userState: userStateReducer,
	},
});
