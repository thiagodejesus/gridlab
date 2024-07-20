pub use grid_engine::engine::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GridEngineWasm {
    grid_engine: GridEngine,
}

#[wasm_bindgen]
pub struct NodeWasm {
    node: Node,
    #[wasm_bindgen(js_name = x)]
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[wasm_bindgen]
impl NodeWasm {
    #[wasm_bindgen(js_name = getId)]
    pub fn get_id(&self) -> String {
        self.node.id.clone()
    }
}

#[wasm_bindgen]
impl GridEngineWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> GridEngineWasm {
        GridEngineWasm {
            grid_engine: GridEngine::new(rows, cols),
        }
    }

    #[wasm_bindgen(js_name = addItem)]
    pub fn add_item(
        &mut self,
        id: String,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Result<String, JsError> {
        match self.grid_engine.add_item(id, x, y, w, h) {
            Ok(id) => Ok(id),
            Err(e) => Err(JsError::new(&e.get_message())),
        }
    }

    #[wasm_bindgen(js_name = moveItem)]
    pub fn move_item(&mut self, id: &str, x: usize, y: usize) -> Result<(), JsError> {
        match self.grid_engine.move_item(id, x, y) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsError::new(&e.get_message())),
        }
    }

    #[wasm_bindgen(js_name = removeItem)]
    pub fn remove_item(&mut self, id: &str) -> Result<(), JsError> {
        match self.grid_engine.remove_item(id) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsError::new(&e.get_message())),
        }
    }

    #[wasm_bindgen(js_name = getNodes)]
    pub fn get_nodes(&self) -> Vec<NodeWasm> {
        let nodes = self.grid_engine.get_nodes();

        nodes
            .iter()
            .map(|node| NodeWasm {
                node: node.clone(),
                x: node.x,
                y: node.y,
                h: node.h,
                w: node.w,
            })
            .collect()
    }

    #[wasm_bindgen(js_name = serializedAsStr)]
    pub fn serialized_as_str(&self) -> String {
        self.grid_engine.serialized_as_str()
    }

    #[wasm_bindgen(js_name = fromSerializedStr)]
    pub fn from_serialized_str(serialized_str: &str) -> Result<GridEngineWasm, JsError> {
        match GridEngine::from_str(serialized_str) {
            Ok(grid_engine) => Ok(GridEngineWasm { grid_engine }),
            Err(e) => Err(JsError::new(&e.get_message())),
        }
    }
}
