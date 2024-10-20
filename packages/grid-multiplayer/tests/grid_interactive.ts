import { EventName, GridEngineWasm } from "wasm-bindings";
import { createInterface } from "node:readline/promises";
import { GridMultiplayerClient } from "../src/grid_multiplayer_client";
import { Logger } from "../src/utils/logger";

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
      const [name, x, y, width, height] = args;

      if (!name || !x || !y || !width || !height) {
        console.error(
          "Invalid arguments, expected: add <name> <x> <y> <width> <height>"
        );
        break;
      }

      grid.addItem(
        name,
        parseInt(x),
        parseInt(y),
        parseInt(width),
        parseInt(height)
      );
      break;
    case "mv":
      const [itemName, newX, newY] = args;
      if (!itemName || !newX || !newY) {
        console.error(
          "Invalid arguments, expected: mv <item_name> <new_x> <new_y>"
        );
        break;
      }
      grid.moveItem(itemName, parseInt(newX), parseInt(newY));
      break;
    case "rm":
      const [itemNameToRemove] = args;
      if (!itemNameToRemove) {
        console.error("Missing item name, expected: rm <item_name>");
        break;
      }
      grid.removeItem(itemNameToRemove);
      break;
    default:
      console.error("Invalid command");
  }
};

const scriptedMode = async () => {
  const gridClient = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
    logger: new Logger("Grid Interactive"),
  });
  const grid = gridClient.grid;

  const commands = [
    "add a 2 2 2 4",
    "add b 4 2 2 4",
    "add c 0 2 2 2",
    "rm b",
    "add d 4 2 2 3",
    "add e 2 2 2 4",
    "add f 2 2 2 4",
    "rm f",
    "add g 2 2 2 4",
    "rm a",
    "mv c 1 0",
    "mv c 2 0",
    "mv c 2 2",
    "mv c 3 2",
    "mv c 4 10",
    "mv c 4 6",
    "mv d 1 1",
    "mv c 4 6", // Bug
  ];
  let i = 1;
  for (const command of commands) {
    i++;
    await sleep(100);
    await handleCommand(grid, command);
  }

  // // Interactive mode
  // await interactiveMode(grid)
};

const interactiveMode = async (grid: GridEngineWasm) => {
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

// const main = async () => {
// scriptedMode();
// };

// main();

const testConcurrency = async () => {
  const gridClient1 = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
    logger: new Logger("GridClient1"),
  });
  const grid1 = gridClient1.grid;

  const gridClient2 = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
    logger: new Logger("GridClient2"),
  });
  const grid2 = gridClient2.grid;

  const gridClient3 = await GridMultiplayerClient.initialize({
    gridOpts: { width: 16, height: 12 },
    url: "http://localhost:3000",
    logger: new Logger("GridClient3"),
  });
  const grid3 = gridClient3.grid;

  const commands = ["add a 0 2 2 4", "add b 2 2 2 4", "add c 4 2 2 4"];
  let i = 1;
  for (const command of commands) {
    i++;
    await sleep(50);
    await handleCommand(grid1, command);
  }

  await sleep(50)
  // Move items in a way that does not collide
  await Promise.all([
    handleCommand(grid2, "mv a 0 0"),
    handleCommand(grid3, "mv b 0 6"),
  ]);
};

testConcurrency();
