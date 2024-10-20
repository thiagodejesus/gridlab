// Writes a websocket client that connects to localhost:3000 and sends Hello World

console.log("running");
import { io } from "socket.io-client";
import { EventName, GridEngineWasm } from "wasm-bindings";
import { GridMultiplayerClient } from "../src/grid_multiplayer_client";
import { Logger } from "../src/utils/logger";

const watchGridChanges = async () => {
  const gridClient = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
    logger: new Logger("GridClient"),
  });
  const grid = gridClient.grid;

  console.log("Received grid");
  console.log(grid.getGridView().getGridFormatted());

  grid.addEventListener(EventName.BatchChange, (g, v) => {
    console.log(g.getGridFormatted());
  });
};

watchGridChanges();
