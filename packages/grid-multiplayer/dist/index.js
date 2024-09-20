"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const grid_multiplayer_server_1 = require("./grid_multiplayer_server");
const main = () => {
    const server = new grid_multiplayer_server_1.GridMultiplayerServer({});
    // server.onConnection((socket) => {
    //   console.log("onConnection 2");
    // });
    server.onChange(() => {
        console.log("HasChanged");
    });
};
main();
