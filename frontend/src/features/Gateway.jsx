import { backendUrl } from "../routes";
import {
	setArenaHeight,
	setArenaWidth,
	setPlayers,
} from "../redux_logic/slices/gameStateSlice";

import store from "../redux_logic/store";

class Gateway {
	constructor() {
		if (Gateway.exists) {
			return Gateway.instance;
		}

		console.debug("Gateway constructor");

		Gateway.exists = true;
		Gateway.instance = this;

		this.started = false;
		this.connected = false;
		this.auto_restart = true;
		this.callbacks = [registerCallback, turnCallback];
	}

	destructor() {
		console.debug("Gateway destructor");
	}

	start() {
		if (!this.started) {
			this.started = true;
			this.connection = new WebSocket(backendUrl);

			this.connection.onopen = this.on_open.bind(this);
			this.connection.onmessage = this.on_message.bind(this);
			this.connection.onclose = this.on_close.bind(this);
			this.connection.onerror = this.on_error.bind(this);
		}
	}

	stop() {
		if (this.started) {
			this.started = false;
			this.auto_restart = false;
			this.connection.close();
		}
	}

	on_open() {
		this.connected = true;

		console.debug("gateway ready");
	}

	on_message(message) {
		console.debug(JSON.parse(message.data));
		console.debug(message.data);
		for (const cb of this.callbacks) {
			cb(JSON.parse(message.data));
		}
	}

	on_close() {
		this.connected = false;

		console.debug("gateway closed");

		if (!this.auto_restart) {
			this.auto_restart = true;
			return;
		}

		setTimeout(() => {
			console.debug("gateway reconnecting");
			this.start();
		}, 3000);
	}

	on_error(error) {
		throw new Error(error);
	}

	send(data) {
		if (!this.connected) {
			return;
		}

		this.connection.send(JSON.stringify(data));
	}

	feed(callback) {
		this.callbacks.push(callback);
	}
}

const registerCallback = (message) => {
	console.debug("Register", "Register" in message);
	if (!("Register" in message)) {
		return;
	}
	console.debug(message["Register"]);
	const width = message["Register"]["field_width"];
	const height = message["Register"]["field_height"];
	console.debug(width, height);
	store.dispatch(setArenaWidth(width));
	store.dispatch(setArenaHeight(height));
};

const turnCallback = (message) => {
	console.debug("Turn", "Turn" in message);
	if (!("Turn" in message)) {
		return;
	}
	console.debug(message["Turn"]);
	const players = message["Turn"]["Players"];
	store.dispatch(setPlayers(players));
};

export default Gateway;
