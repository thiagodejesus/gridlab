pub use grid_engine::grid_engine::*;
pub use grid_engine::grid_view::*;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[wasm_bindgen]
/// Some types for the TS bindings.
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "(gridEngine: GridViewWasm, value: EventValue) => void")]
    pub type EventListenerCallback;

    #[wasm_bindgen(typescript_type = "Change[]")]
    #[derive(Debug)]
    pub type Changes;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct GridViewWasm {
    grid_view: GridView,
}

#[wasm_bindgen]
impl GridViewWasm {
    pub(crate) fn from_grid_view(grid_view: &GridView) -> GridViewWasm {
        GridViewWasm {
            grid_view: grid_view.clone(),
        }
    }

    #[wasm_bindgen(js_name = getNodes)]
    pub fn get_nodes(&self) -> Vec<Node> {
        self.grid_view.get_nodes()
    }

    #[wasm_bindgen(js_name = printGrid)]
    pub fn print_grid(&self) {
        self.grid_view.print_grid();
    }

    #[wasm_bindgen(js_name = getGridFormatted)]
    pub fn get_grid_formatted(&self) -> String {
        self.grid_view.get_grid_formatted()
    }

    #[wasm_bindgen(js_name = serializedAsStr)]
    pub fn serialized_as_str(&self) -> String {
        self.grid_view.serialized_as_str()
    }

    #[wasm_bindgen(js_name = hash)]
    pub fn hash(&self) -> String {
        self.grid_view.hash()
    }
}

#[wasm_bindgen]
pub struct GridEngineWasm {
    grid_engine: GridEngine,
}

#[wasm_bindgen]
impl GridEngineWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> GridEngineWasm {
        console_error_panic_hook::set_once();
        
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

    #[wasm_bindgen(js_name = getGridView)]
    pub fn get_grid_view(&self) -> GridViewWasm {
        GridViewWasm::from_grid_view(&self.grid_engine.get_grid_view())
    }

    #[wasm_bindgen(js_name = getNodes)] // Should remove this as this can be done via getGridView
    pub fn get_nodes(&self) -> Vec<Node> {
        self.grid_engine.get_grid_view().get_nodes()
    }

    #[wasm_bindgen(js_name = getGridFormatted)] // Should remove this as this can be done via getGridView
    pub fn get_grid_formatted(&self) -> String {
        self.grid_engine.get_grid_view().get_grid_formatted()
    }

    #[wasm_bindgen(js_name = applyChanges)]
    pub fn apply_changes(&mut self, changes: Changes) -> Result<(), JsError> {
        log(&format!("Args received, {:?}", changes));
        let changes: Vec<Change> = serde_wasm_bindgen::from_value(changes.obj)?;
        log(&format!("Changes parsed, {:?}", changes));
        self.grid_engine.apply_changes(&changes);
        Ok(())
    }

    #[wasm_bindgen(js_name = serializedAsStr)] // Should remove this as this can be done via getGridView
    pub fn serialized_as_str(&self) -> String {
        self.grid_engine.get_grid_view().serialized_as_str()
    }

    #[wasm_bindgen(js_name = fromSerializedStr)]
    pub fn from_serialized_str(serialized_str: &str) -> Result<GridEngineWasm, JsError> {
        match GridEngine::from_str(serialized_str) {
            Ok(grid_engine) => Ok(GridEngineWasm {
                grid_engine: grid_engine,
            }),
            Err(e) => Err(JsError::new(&e.get_message())),
        }
    }

    #[wasm_bindgen(js_name = addEventListener)]
    pub fn add_event_listener(
        &mut self,
        event_name: EventName,
        listener_callback: EventListenerCallback,
    ) {
        self.grid_engine.events.add_listener(
            event_name.clone(),
            Box::new(move |grid, event_value| {
                let this = JsValue::null();

                log(&format!(
                    "Event received, {:?}, {:?}",
                    event_name, event_value
                ));

                // let formatted = self.get_grid_formatted();
                let grid_view = JsValue::from(GridViewWasm::from_grid_view(grid));

                listener_callback
                    .call2(
                        &this,
                        &grid_view,
                        &serde_wasm_bindgen::to_value(event_value)
                            .expect("Failed to parse event_value"),
                    )
                    .expect("Failed to call listener_callback");
            }),
        );
    }
}
