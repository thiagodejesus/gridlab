// Should change to fastify
import {
  GridEngineWasm,
  EventName,
  GridViewWasm,
  EventValue,
} from "wasm-bindings";
import { io, Socket } from "socket.io-client";

type InitializeGridMultiplayerClientOpts = {
  url: string;
  gridOpts: {
    width: number;
    height: number;
  };
};

type GridMultiplayerClientOpts = {
  url: string;
  grid: GridEngineWasm;
  socket: Socket;
};

export class GridMultiplayerClient {
  private url: string;
  grid: GridEngineWasm;
  private socket: Socket;

  constructor(opts: GridMultiplayerClientOpts) {
    this.url = opts.url;
    this.grid = opts.grid;
    this.socket = opts.socket;
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

      // socket.on("connect", () => {
      //   console.log("connected");
      //   socket.emit("message", "Hello World");
      // });

      // socket.on("message", (data: any) => {
      //   console.log("message received");
      //   console.log(data);
      // });

      let grid: GridEngineWasm;

      socket.on("grid", (data: string) => {
        console.log("grid received");
        console.log(data);
        grid = GridEngineWasm.fromSerializedStr(data);
        clearTimeout(timeout);
        grid.addEventListener(EventName.BatchChange, (_, event) => {
          // console.log("Grid change emitted socket change", event.value);
          socket.emit("changes", event.value, socket.id);
        });
        resolve(new GridMultiplayerClient({ grid, url: opts.url, socket }));
      });

      socket.on("changes", (changes: EventValue['value']) => {
        console.log("Socket Received Change", changes);
        const actualHash = grid.getGridView().hash();
        if (changes.hash_before === actualHash) {
          grid.applyExternalChanges(changes.changes);
        } else if (changes.hash_after === actualHash) {
          // Duplicated
          console.log("Received already applied change");
        } else {
          // Force resync
        }
      });
    });
  }

  getGridView(): GridViewWasm {
    return this.grid.getGridView();
  }
}
