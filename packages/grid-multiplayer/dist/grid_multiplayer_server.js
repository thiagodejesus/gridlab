"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.GridMultiplayerServer = void 0;
// Should change to fastify
const wasm_bindings_1 = require("wasm-bindings");
const express_1 = __importDefault(require("express"));
const node_http_1 = require("node:http");
const socket_io_1 = require("socket.io");
class GridMultiplayerServer {
    rooms;
    server;
    constructor(opts) {
        this.rooms = {};
        const app = (0, express_1.default)();
        const server = (0, node_http_1.createServer)(app);
        const io = new socket_io_1.Server(server);
        io.listen(3000);
        console.log("Server is running on port 3000");
        this.server = io;
        this.server.on("connection", (socket) => {
            console.log("headers", socket.handshake.headers);
            // Should add a entity validator like Joi
            if (!socket.handshake.headers["grid-id"]) {
                console.log("invalid connection, missing gridId");
                socket.disconnect();
                return;
            }
            const gridID = socket.handshake.headers["grid-id"];
            const rooms = this.rooms;
            if (!rooms[gridID]) {
                rooms[gridID] = {
                    id: gridID,
                    grid: new wasm_bindings_1.GridEngineWasm(16, 12),
                    connections: [
                        { connection: socket.conn, user: { id: "a", name: "user" } },
                    ],
                };
            }
            else {
                rooms[gridID].connections.push({
                    connection: socket.conn,
                    user: { id: "b", name: "user" },
                });
            }
            // Sends the grid as bytes
            const grid = rooms[gridID].grid;
            console.log("Rooms", rooms);
            console.log("Grid", grid);
            grid.addEventListener(wasm_bindings_1.EventName.Change, (grid, event) => {
                console.log("Grid changed");
                console.log("Grid ", grid);
                console.log("Event ", event);
            });
            grid.addEventListener(wasm_bindings_1.EventName.BatchChange, (g, event) => {
                console.log("Grid Batch changed");
                console.log(g.getGridFormatted());
            });
            socket.on("change", (change) => {
                console.log("Socket Received Change", change);
                grid.applyExternalChanges([change]);
            });
            socket.emit("grid", grid.serializedAsStr());
            // console.log("rooms", rooms);
            // console.log("a user connected");
        });
    }
    onConnection(fn) {
        this.server.on("connection", fn);
    }
    onChange(fn) {
        Object.entries(this.rooms).forEach(([_, room]) => {
            console.log("Apply onChange on grids");
            // room.grid.on
        });
    }
}
exports.GridMultiplayerServer = GridMultiplayerServer;
