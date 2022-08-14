import { backendUrl } from '../routes'
class Gateway {
    constructor () {
        if(Gateway.exists) {
            return Gateway.instance;
        }

        Gateway.exists = true;
        Gateway.instance = this;

        this.connected = false;
        this.auto_restart = true;
        this.callbacks = [];

    }

    start() {
        this.connection = WebSocket(backendUrl);

        this.connection.onopen
    }

    stop() {
        this.auto_restart = false;
        this.connection.close();
    }

    on_open() {
        this.connected = true;

        console.debug('gateway ready');
    }

    on_message(message) {
        for(const cb of this.callbacks) {
            cb(message.data);
        }
    }

    on_close() {
        this.connected = false;

        console.debug('gateway closed');

        if(!this.auto_restart) {
            this.auto_restart = true;
            return;
        }

        setTimeout(() => {
            console.debug('gateway reconnecting');
            this.start();
        }, 3000);
    }

    on_error(error) {
        throw new Error(error);
    }

    send(data) {
        if(!this.connected) {
            return;
        }

        this.connection.send(JSON.stringify(data));
    }

    feed(callback) {
        this.callbacks.push(callback);
    }
}

export default Gateway;