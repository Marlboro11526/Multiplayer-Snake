import { configureStore } from '@reduxjs/toolkit'
import gameStateReducer from './gameStateSlice';
import userStateReducer from './userSlice';

export default configureStore({
  reducer: {
    gameState: gameStateReducer,
    userState: userStateReducer,
  },
});