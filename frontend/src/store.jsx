import { configureStore } from '@reduxjs/toolkit'
import gameStateReducer from './reducers/gameStateSlice';
import userStateReducer from './reducers/userSlice';

export default configureStore({
  reducer: {
    gameState: gameStateReducer,
    userState: userStateReducer,
  },
});