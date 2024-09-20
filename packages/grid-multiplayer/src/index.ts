import { GridMultiplayerServer } from "./grid_multiplayer_server";

const main = () => {
  const server = new GridMultiplayerServer({});
  // server.onConnection((socket) => {
  //   console.log("onConnection 2");
  // });
  server.onChange(() => {
    console.log("HasChanged");
  });
};

main();
