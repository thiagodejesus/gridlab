"use client";

import React, { useRef, useEffect, useState, useCallback } from "react";
import { GridStack, GridStackWidget } from "gridstack";
import "gridstack/dist/gridstack.min.css";
import dynamic from "next/dynamic";
import { instantiate, gridLabTypes } from "gridlab-ts-web";

// import { Hello } from "./components/hello";

import { v4 as uuidv4 } from "uuid";

export default function Home() {
  const [nodes, setNodes] = useState<any>([]);
  const gridStack = useRef<GridStack>();
  const gridLabEngine = useRef<gridLabTypes.GridEngineWasm>();

  useEffect(() => {
    (async () => {
      const { GridEngineWasm } = await instantiate();
      // GridStack.registerEngine(CustomEngine)
      gridStack.current = GridStack.init({
        cellHeight: 50,
        float: true,
        column: 12,
        minRow: 12,
        // subGridDynamic: false,
        // acceptWidgets: false,
        disableResize: false,
        // sizeToContent: true,
        draggable: {
          // pause: 100,
        },
      });
      gridLabEngine.current = new GridEngineWasm(12, 12);

      // WE ONLY CARE ABOUT POSITION CHANGES
      gridStack.current.on("change", function (event, items) {
        console.log("MAIN GRID", event, items);
      });

      gridStack.current.on(
        "dropped",
        function (event, previousWidget, newWidget) {
          console.log("previous widget", previousWidget);
          console.log("new Widget", newWidget);
          if (!previousWidget) {
            // gridStack.current?.removeWidget(newWidget.el as any, true, false);
          }
        }
      );

      // WE DO NOT EXPECT THIS EVENT
      gridStack.current.on("added", function (event, items) {
        try {
          console.log("adding gridlab item");
          gridLabEngine.current?.addItem(
            items[0].id as string,
            items[0].x as number,
            items[0].y as number,
            items[0].w as number,
            items[0].h as number
          );
        } catch (e) {
          console.log("Error adding gridlab item", e);
        }
        console.log("added", event, items);
      });

      gridStack.current.on("removed", function (event, items) {
        try {
          console.log("not removing gridlab item");
          // gridLabEngine.current?.removeItem(
          //   items[0].x as number,
          //   items[0].y as number
          // );
        } catch (e) {
          console.log("Error removing gridlab item", e);
        }
      });
    })();

    return () => {
      console.log("CLEANING UP MAIN GRID");
    };
  }, []);

  useEffect(() => {
    nodes.forEach((node: any) => {
      const curNode = gridStack.current?.engine.nodes.find(
        (gsNode) => gsNode.id === node.id
      );

      if (curNode) {
        gridStack.current?.update(curNode.el as any, {
          x: node.x || 0,
          y: node.y || 0,
          w: node.w || 1,
          h: node.h || 1,
        });
      }
    });
  }, [nodes]);

  const handleAddComponent = () => {
    const id = uuidv4();
    const newNode = {
      x: 0,
      y: 0,
      w: 2,
      h: 2,
      autoPosition: false,
      body: {
        component: {
          id,
          name: "hello",
        },
      },
      id,
    };
    gridStack.current?.addWidget({
      x: 0,
      y: 0,
      w: 2,
      h: 2,
      autoPosition: false,
      id,
      content: "hello",
    });
    // setNodes((prevNodes: any) => [...prevNodes, newNode]);
  };

  const handleSave = () => {
    const save = gridStack.current?.save(false, true);
    console.log(save);
  };

  const handleClear = () => {
    // nodes.forEach((node: any) => {
    //   gridStack.current?.removeWidget(node.id, false);
    // });
    // setNodes([]);
    gridStack.current?.removeAll(true);
  };

  const handleGetGridlabItems = () => {
    const gridWasmNodes = gridLabEngine.current?.getNodes()?.map((i) => {
      const obj: any = {};
      const keys = ["x", "y", "w", "h"];
      keys.forEach((k) => {
        const value: any = (i as any)[k];
        obj[k] = value;
      });

      return {
        ...obj,
        id: i.getId(),
      };
    });
    console.log(gridWasmNodes);
  };

  return (
    <div>
      <button onClick={handleAddComponent}>Add Component</button>
      <button onClick={handleSave}>Save</button>
      <button onClick={handleClear}>Clear</button>
      <button onClick={handleGetGridlabItems}>Get Gridlab Items</button>
      <div className="grid-stack">
        {/* {nodes.map((node: any) => {
        console.log("node", node);

        return (
          <Hello
            node={node}
            mainGridRef={gridStack.current as GridStack}
            key={node.body.component.id}
          />
        );
      })} */}
      </div>
    </div>
  );
}
