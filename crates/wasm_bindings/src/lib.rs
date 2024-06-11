pub use grid_engine::grid_engine::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GridEngineWasm {
    grid_engine: GridEngine,
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
    pub fn add_item(&mut self, x: usize, y: usize, w: usize, h: usize) -> Result<String, JsError> {
        match self.grid_engine.add_item(x, y, w, h) {
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
}
