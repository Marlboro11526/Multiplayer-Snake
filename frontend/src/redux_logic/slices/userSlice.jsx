import { createSlice } from "@reduxjs/toolkit";

export const userSlice = createSlice({
	name: "user_state",
	initialState: {
		name: null,
		uuid: null,
	},
	reducers: {
		setName: (state, action) => {
			return {
				...state,
				name: action.payload,
			};
		},
		setUuid: (state, action) => {
			return {
				...state,
				uuid: action.payload,
			};
		},
	},
});

export const { setName, setUuid } = userSlice.actions;

export default userSlice.reducer;
