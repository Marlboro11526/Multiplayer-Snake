import { backendUrl } from '../routes'
export class Gateway {

    constructor () {
        if(Gateway.exists) {
            return Gateway.instance;
        }

        Gateway.exists = true;
        Gateway.instance = this;

        this.started = false;
        this.connected = false;
        this.auto_restart = true;
        this.callbacks = [];
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

        console.debug('gateway ready');
    }

    on_message(message) {
        console.debug(message);
        console.debug(JSON.parse(message.data));
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