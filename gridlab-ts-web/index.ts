import * as gridLab from "grid-lab-web";
import type * as gridLabTypes from "grid-lab-web";

export const instantiate = async () => {
  const { default: init, initSync: _, ...lib } = gridLab;

  await init();
  return lib;
};

export default instantiate;
export type { gridLabTypes };
