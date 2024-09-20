import { EventName, GridEngineWasm } from "wasm-bindings";
import { createInterface } from "node:readline/promises";
import { GridMultiplayerClient } from "../src/grid_multiplayer_client";

const printInterface = (grid: GridEngineWasm) => {
  console.clear();
  console.log("Grid interface:");
  printGrid(grid);
};

const printGrid = (grid: GridEngineWasm) => {
  const formatted = grid.getGridFormatted();
  console.log(formatted);
};

const sleep = (ms: number) => new Promise((res) => setTimeout(res, ms));

const handleCommand = async (grid: GridEngineWasm, command: string) => {
  const [action, ...args] = command.split(" ");
  switch (action) {
    case "add":
      console.group("Add");
      const [name, x, y, width, height] = args;

      if (!name || !x || !y || !width || !height) {
        console.log(
          "Invalid arguments, expected: add <name> <x> <y> <width> <height>"
        );
        console.groupEnd();
        break;
      }

      grid.addItem(
        name,
        parseInt(x),
        parseInt(y),
        parseInt(width),
        parseInt(height)
      );
      console.groupEnd();
      break;
    case "mv":
      console.group("Move");
      const [itemName, newX, newY] = args;
      if (!itemName || !newX || !newY) {
        console.log(
          "Invalid arguments, expected: mv <item_name> <new_x> <new_y>"
        );
        console.groupEnd();
        break;
      }
      grid.moveItem(itemName, parseInt(newX), parseInt(newY));
      console.groupEnd();
      break;
    case "rm":
      console.group("Remove");
      const [itemNameToRemove] = args;
      if (!itemNameToRemove) {
        console.log("Missing item name, expected: rm <item_name>");
        console.groupEnd();
        break;
      }
      grid.removeItem(itemNameToRemove);
      console.groupEnd();
      break;
    default:
      console.log("Invalid command");
  }
};

const scriptedMode = async () => {
  const gridClient = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
  });
  const grid = gridClient.grid;
  // grid.addEventListener(EventName.BatchChange, (event) => {
  //   console.log("Change", event);
  // });

  const commands = [
    "add a 2 2 2 4 1",
    "add b 4 2 2 4 2",
    "add c 0 2 2 2",
    "rm b",
    "add d 4 2 2 3 0",
    "add e 2 2 2 4 1",
    "add f 2 2 2 4 1",
    "rm f",
    "add g 2 2 2 4 1",
    "rm a",
    "mv c 1 0",
    "mv c 2 0",
    "mv c 2 2",
    "mv c 3 2",
  ];
  for (const command of commands) {
    await handleCommand(grid, command);
    await sleep(1250);
    // printInterface(grid);
  }
  // Interactive mode
  while (true) {
    printInterface(grid);
    // Sleep for .100
    await sleep(100);

    // Get input on Node
    const rl = createInterface({
      input: process.stdin,
      output: process.stdout,
    });
    const answer = await rl.question("Enter command: ");
    rl.close();
    await handleCommand(grid, answer);
  }
};

const main = async () => {
  scriptedMode();
};

main();
