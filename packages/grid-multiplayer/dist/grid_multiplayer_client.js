"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GridMultiplayerClient = void 0;
// Should change to fastify
const wasm_bindings_1 = require("wasm-bindings");
const socket_io_client_1 = require("socket.io-client");
class GridMultiplayerClient {
    url;
    grid;
    socket;
    constructor(opts) {
        this.url = opts.url;
        this.grid = opts.grid;
        this.socket = opts.socket;
    }
    static async initialize(opts) {
        return new Promise((resolve, reject) => {
            const socket = (0, socket_io_client_1.io)(opts.url, {
                extraHeaders: {
                    "grid-id": "123",
                },
            });
            const timeout = setTimeout(() => {
                reject("timeout");
            }, 5000);
            // socket.on("connect", () => {
            //   console.log("connected");
            //   socket.emit("message", "Hello World");
            // });
            // socket.on("message", (data: any) => {
            //   console.log("message received");
            //   console.log(data);
            // });
            let grid;
            socket.on("grid", (data) => {
                console.log("grid received");
                console.log(data);
                console.log("BeforeError");
                grid = wasm_bindings_1.GridEngineWasm.fromSerializedStr(data);
                console.log("AfterError");
                clearTimeout(timeout);
                grid.addEventListener(wasm_bindings_1.EventName.Change, (_, event) => {
                    console.log("Grid change emitted socket change");
                    socket.emit("change", event.value);
                });
                resolve(new GridMultiplayerClient({ grid, url: opts.url, socket }));
            });
            socket.on("change", (data) => {
                console.log("change", data);
            });
        });
    }
}
exports.GridMultiplayerClient = GridMultiplayerClient;
