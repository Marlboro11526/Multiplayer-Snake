import { createSlice } from '@reduxjs/toolkit'

export const userSlice = createSlice({
    name: 'user_state',
    initialState: {
        name: "bartek",
        uuid: null,
    },
    reducers: {
        setName: (state, action) => {
            return {
                name : action.payload,
                ...state
            }
        },
        setUuid: (state, action) => {
            return {
                uuid: action.payload,
                ...state,
            }
        },
    },
});

export const { setName, setUuid } = userSlice.actions;

export default userSlice.reducer;