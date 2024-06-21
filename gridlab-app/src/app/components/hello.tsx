import { GridStack } from "gridstack";
import React, { useEffect, useRef } from "react";
import { createPortal } from "react-dom";

export function Hello({
  mainGridRef,
  node,
}: {
  mainGridRef: GridStack;
  node: any;
}) {
  console.log("DEBUG RENDERING COMPONENT", node);
  const buttonRef = useRef();

  useEffect(() => {
    if (!mainGridRef) return;

    console.log("CRIANDO node", node);
    mainGridRef.makeWidget(`#${node.id}`);

    return () => {
      console.log("CLEANING UP");
    };
  }, [mainGridRef, node.id]);

  const handleClick = () => {
    console.log("CLICKED", buttonRef);
  };

  return (
    <>
      {!mainGridRef ? (
        <div>Loading ...</div>
      ) : (
        createPortal(
          <div
            className="grid-stack-item component"
            gs-x={node?.x}
            gs-y={node?.y}
            gs-w={node?.w}
            gs-h={node?.h}
            gs-id={node.id}
            id={node.id}
          >
            <div className="grid-stack-item-content component">
              <button ref={buttonRef as any} onClick={handleClick}>
                Hello
              </button>
              <input />
            </div>
          </div>,
          mainGridRef.el
        )
      )}
    </>
  );
}
