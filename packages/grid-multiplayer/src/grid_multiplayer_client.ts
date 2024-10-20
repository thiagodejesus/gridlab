// Should change to fastify
import {
  GridEngineWasm,
  EventName,
  GridViewWasm,
  EventValue,
} from "wasm-bindings";
import { io, Socket } from "socket.io-client";
import { Logger } from "./utils/logger";

type InitializeGridMultiplayerClientOpts = {
  url: string;
  gridOpts: {
    width: number;
    height: number;
  };
  logger: Logger;
};

type GridMultiplayerClientOpts = {
  url: string;
  grid: GridEngineWasm;
  socket: Socket;
  logger: Logger;
};

export class GridMultiplayerClient {
  private url: string;
  grid: GridEngineWasm;
  private socket: Socket;
  private logger: Logger = new Logger("GridMultiplayerClient");

  constructor(opts: GridMultiplayerClientOpts) {
    this.url = opts.url;
    this.grid = opts.grid;
    this.socket = opts.socket;
    this.logger = opts.logger;
  }

  static async initialize(
    opts: InitializeGridMultiplayerClientOpts
  ): Promise<GridMultiplayerClient> {
    return new Promise((resolve, reject) => {
      const socket = io(opts.url, {
        extraHeaders: {
          "grid-id": "123",
        },
      });

      const timeout = setTimeout(() => {
        reject("timeout");
      }, 5000);

      let grid: GridEngineWasm;

      socket.on("grid", (data: string) => {
        opts.logger.info("grid received");
        grid = GridEngineWasm.fromSerializedStr(data);
        clearTimeout(timeout);
        grid.addEventListener(EventName.BatchChange, (_, event) => {
          opts.logger.debug("Sending Changes");
          socket.emit("changes", event.value, socket.id);
        });
        resolve(
          new GridMultiplayerClient({
            grid,
            url: opts.url,
            socket,
            logger: opts.logger,
          })
        );
      });

      socket.on("changes", (changes: EventValue["value"]) => {
        const actualHash = grid.getGridView().hash();
        opts.logger.info("Changes received");
        opts.logger.info(
          JSON.stringify({
            hashBefore: changes.hash_before,
            hashAfter: changes.hash_after,
            actualHash,
            changes: changes.changes,
          })
        );
        console.log(grid.getGridView().getGridFormatted());
        if (
          changes.hash_before === actualHash &&
          changes.hash_after !== actualHash
        ) {
          opts.logger.info("Applying change");
          grid.applyChanges(changes.changes);
        } else if (changes.hash_after === actualHash) {
          // Duplicated
          opts.logger.warn("Already applied change");
        } else {
          // Force resync
          opts.logger.error("Should Resync");
        }
      });
    });
  }

  getGridView(): GridViewWasm {
    return this.grid.getGridView();
  }
}
