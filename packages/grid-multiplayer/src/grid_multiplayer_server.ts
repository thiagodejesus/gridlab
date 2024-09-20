// Should change to fastify
import { GridEngineWasm, EventName, EventValue } from "wasm-bindings";
import express from "express";
import { createServer } from "node:http";
import { Server, Socket } from "socket.io";

type User = {
  id: string;
  name: string;
};

type Connection = {
  user: User;
  socket: Socket;
};

type Room = {
  id: string;
  grid: GridEngineWasm;
  connections: Connection[];
};

type GridMultiplayerServerOptions = {};

type OnConnection = (socket: Socket) => void;
type OnChange = () => void;

export class GridMultiplayerServer {
  private rooms: { [key: string]: Room };
  private server: Server;

  constructor(opts: GridMultiplayerServerOptions) {
    this.rooms = {};

    const app = express();
    const server = createServer(app);
    const io = new Server(server);

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

      const gridID = socket.handshake.headers["grid-id"] as string;
      const rooms = this.rooms;
      if (!rooms[gridID]) {
        rooms[gridID] = {
          id: gridID,
          grid: new GridEngineWasm(16, 12),
          connections: [{ socket, user: { id: "a", name: "user" } }],
        };
      } else {
        rooms[gridID].connections.push({
          socket,
          user: { id: "b", name: "user" },
        });
      }

      // Sends the grid as bytes

      const grid = rooms[gridID].grid;

      console.log("Rooms", rooms);
      console.log("Grid", grid);

      grid.addEventListener(EventName.BatchChange, (g, event) => {
        console.log("Grid Batch changed", event.value);
        console.log(g.getGridFormatted());
        rooms[gridID].connections.forEach((c) => {
          console.log("Sending to ", c.socket.id);
          c.socket.emit("changes", event.value);
        });
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

      socket.emit("grid", grid.serializedAsStr());
      // console.log("rooms", rooms);
      // console.log("a user connected");
    });
  }

  onConnection(fn: OnConnection) {
    this.server.on("connection", fn);
  }

  onChange(fn: OnChange) {
    Object.entries(this.rooms).forEach(([_, room]) => {
      console.log("Apply onChange on grids");
      // room.grid.on
    });
  }
}
