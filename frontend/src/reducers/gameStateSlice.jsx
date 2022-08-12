import { createSlice } from '@reduxjs/toolkit'

export const gameSlice = createSlice({
    name: 'game_state',
    initialState: {
        payers: {},
        arena_width: 10,
        arena_height: 15,
    },
    reducers: {
        setPlayers: (state, action) => {
            return {
                ...state,
                players: action.payload,
            };
        },
        setArenaWidth: (state, action) => {
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
    },
});

export const { setPlayers, setArenaWidth, setArenaHeight } = gameSlice.actions

export default gameSlice.reducer;