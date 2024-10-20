use crate::grid_view::GridView;
use crate::{engine_events::EventListener, error::GridError};
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{collections::BTreeMap, fmt::Debug};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

// TODO, remove unnecessary clones
// TODO, Handle all `expect` and `unwrap` properly
// TODO, set wasm as a optional feature

fn for_cell(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    callback: &mut impl FnMut(usize, usize) -> Result<(), GridError>,
) -> Result<(), GridError> {
    for x in x..x + w {
        for y in y..y + h {
            callback(x, y)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
enum UpdateGridOperation {
    Add,
    Remove,
}

fn update_grid(
    grid: &mut Grid<Option<String>>,
    node: &Node,
    x: usize,
    y: usize,
    operation: UpdateGridOperation,
) -> Result<(), GridError> {
    let element_at_position = grid.get_mut(y, x);

    match element_at_position {
        Some(cell) => {
            match operation {
                UpdateGridOperation::Add => {
                    *cell = Some(node.id.to_string());
                }
                UpdateGridOperation::Remove => {
                    if *cell == Some(node.id.to_string()) {
                        *cell = None;
                    }
                }
            }
            Ok(())
        }
        None => Err(GridError::new(
            &format!("Error updating grid with {:?} operation", operation),
            &format!("No element at position X:{x},Y:{y} in grid: {:?}", grid),
            None,
        )),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct Node {
    #[wasm_bindgen(skip)]
    pub id: String,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[wasm_bindgen]
impl Node {
    fn new(id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        Node { id, x, y, w, h }
    }

    #[wasm_bindgen(js_name = getId)]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    fn for_cell(
        &self,
        callback: &mut impl FnMut(usize, usize) -> Result<(), GridError>,
    ) -> Result<(), GridError> {
        for_cell(self.x, self.y, self.w, self.h, callback)
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Tsify)]
pub struct AddChangeData {
    #[wasm_bindgen(skip)]
    pub value: Node,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Tsify)]
pub struct RemoveChangeData {
    #[wasm_bindgen(skip)]
    pub value: Node,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Tsify)]
pub struct MoveChangeData {
    #[wasm_bindgen(skip)]
    pub old_value: Node,
    #[wasm_bindgen(skip)]
    pub new_value: Node,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Tsify)]
#[serde(tag = "type", content = "value")]
pub enum Change {
    Add(AddChangeData),
    Remove(RemoveChangeData),
    Move(MoveChangeData),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Tsify)]
pub struct BatchChangeValue {
    pub changes: Vec<Change>,
    pub hash_before: String,
    pub hash_after: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", content = "value")]
pub enum EventValue {
    BatchChange(BatchChangeValue),
}

// Just for test, should correctly implement display
impl Display for EventValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventValue::BatchChange(value) => write!(f, "BatchChange: {:?}", value),
        }
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone)]
pub enum EventName {
    BatchChange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridEngine {
    pub(crate) grid: Grid<Option<String>>,
    pub(crate) items: BTreeMap<String, Node>,
    #[serde(skip)]
    pending_changes: Vec<Change>,
    #[serde(skip)]
    pub events: EventListener<EventName, EventValue>,
}

impl GridEngine {
    pub fn new(rows: usize, cols: usize) -> GridEngine {
        GridEngine {
            grid: Grid::new(rows, cols),
            items: BTreeMap::new(),
            pending_changes: Vec::new(),
            events: EventListener::default(),
        }
    }

    fn from_grid_view(grid_view: GridView) -> GridEngine {
        GridEngine {
            grid: grid_view.grid.clone(),
            items: grid_view.items.clone(),
            pending_changes: Vec::new(),
            events: EventListener::default(),
        }
    }

    pub fn from_str(serialized: &str) -> Result<GridEngine, GridError> {
        let grid_view: GridView = match serde_json::from_str(serialized) {
            Ok(grid_view) => grid_view,
            Err(err) => {
                println!("Error deserializing GridView {:?}", err);
                return Err(GridError::new(
                    "Error deserializing GridView",
                    "",
                    Some(Box::new(err)),
                ));
            }
        };

        return Ok(GridEngine::from_grid_view(grid_view));
    }

    fn new_node(&mut self, id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        let node = Node::new(id, x, y, w, h);
        node
    }

    fn create_add_change(&mut self, node: &Node) {
        self.pending_changes.push(Change::Add(AddChangeData {
            value: node.clone(),
        }));
    }

    pub fn add_item(
        &mut self,
        id: String,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Result<String, GridError> {
        if self.items.get(&id).is_some() {
            return Err(GridError::new("Id already exists", "", None));
        };

        let node = self.new_node(id, x, y, w, h);
        let node_id = node.id.to_string();

        self.handle_collision(&node, x, y, &self.grid.clone());

        self.create_add_change(&node);

        self.apply_changes(&self.pending_changes.clone());
        self.pending_changes.clear();

        Ok(node_id)
    }

    fn create_remove_change(&mut self, node: &Node) {
        self.pending_changes.push(Change::Remove(RemoveChangeData {
            value: node.clone(),
        }));
    }

    pub fn remove_item(&mut self, id: &str) -> Result<(), GridError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridError::new("Item not found", "", None))?,
        }
        .clone();

        self.create_remove_change(&node);

        self.apply_changes(&self.pending_changes.clone());
        self.pending_changes.clear();
        Ok(())
    }

    fn will_collides_with(
        &self,
        node: &Node,
        x: usize,
        y: usize,
        grid: &Grid<Option<String>>,
    ) -> Vec<String> {
        let mut collides_with = Vec::new();
        for_cell(x, y, node.w, node.h, &mut |x, y| {
            if let Some(cell) = grid.get(y, x) {
                if cell.is_some() {
                    let cell_ref = cell.as_ref().expect("Failed to get cell ref");
                    if cell_ref != &node.id && !collides_with.contains(&cell_ref.to_string()) {
                        collides_with.push(cell_ref.to_string());
                    }
                }
            }
            Ok(())
        })
        .expect("UnhandledError");

        collides_with
    }

    fn handle_collision(&mut self, node: &Node, x: usize, y: usize, grid: &Grid<Option<String>>) {
        let collides_with = self.will_collides_with(node, x, y, grid);
        if collides_with.len() > 0 {
            for collided_id in collides_with {
                let collided = self
                    .items
                    .get(&collided_id)
                    .expect("Failed to get collided node")
                    .clone();
                // Will pass here a grid in which the node is already moved
                let mut new_grid = self.grid.clone();
                node.for_cell(&mut |x, y| {
                    return update_grid(&mut new_grid, node, x, y, UpdateGridOperation::Remove);
                })
                .expect("UnhandledError");

                self.create_move_change(&collided, collided.x, y + node.h, &mut new_grid);
            }
        }
    }

    fn create_move_change(
        &mut self,
        node: &Node,
        new_x: usize,
        new_y: usize,
        grid: &Grid<Option<String>>,
    ) {
        let old_node = node.clone();

        self.handle_collision(node, new_x, new_y, grid);

        self.pending_changes.push(Change::Move(MoveChangeData {
            old_value: old_node,
            new_value: Node::new(node.id.to_string(), new_x, new_y, node.w, node.h),
        }));
    }

    pub fn move_item(&mut self, id: &str, new_x: usize, new_y: usize) -> Result<(), GridError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridError::new("Item not found", "", None))?,
        };

        self.create_move_change(&node.clone(), new_x, new_y, &self.grid.clone());

        self.apply_changes(&self.pending_changes.clone());
        self.pending_changes.clear();

        Ok(())
    }

    pub fn apply_changes(&mut self, changes: &Vec<Change>) {
        let hash_before = self.get_grid_view().hash();
        for change in changes.iter() {
            match &change {
                Change::Add(data) => {
                    let node = &data.value;

                    node.for_cell(&mut |x, y| {
                        return update_grid(&mut self.grid, node, x, y, UpdateGridOperation::Add);
                    })
                    .expect("UnhandledError");

                    self.items.insert(node.id.to_string(), node.clone());
                }
                Change::Remove(data) => {
                    let node = &data.value;

                    node.for_cell(&mut |x, y| {
                        return update_grid(
                            &mut self.grid,
                            node,
                            x,
                            y,
                            UpdateGridOperation::Remove,
                        );
                    })
                    .expect("UnhandledError");

                    self.items.remove(&node.id);
                }
                Change::Move(data) => {
                    let node = &data.new_value;
                    let old_node = &data.old_value;

                    old_node
                        .for_cell(&mut |x, y| {
                            return update_grid(
                                &mut self.grid,
                                old_node,
                                x,
                                y,
                                UpdateGridOperation::Remove,
                            );
                        })
                        .expect("UnhandledError");

                    self.items.insert(node.id.to_string(), node.clone());
                    node.for_cell(&mut |x, y| {
                        return update_grid(&mut self.grid, node, x, y, UpdateGridOperation::Add);
                    })
                    .expect("UnhandledError");
                }
            }
        }
        let grid_view = GridView::new(self);

        self.events.trigger_event(
            &grid_view,
            EventName::BatchChange,
            EventValue::BatchChange(BatchChangeValue {
                changes: changes.iter().map(|change| change.clone()).collect(),
                hash_before,
                hash_after: grid_view.hash(),
            }),
        );
    }

    pub fn get_grid_view(&self) -> GridView {
        GridView::new(self)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_for_cell() {
        let mut results = Vec::new();
        let mut callback = |x: usize, y: usize| {
            results.push((x, y));
            Ok(())
        };

        for_cell(1, 2, 2, 2, &mut callback).unwrap();

        assert_eq!(results, vec![(1, 2), (1, 3), (2, 2), (2, 3)]);
    }

    #[test]
    fn test_add_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();

        assert!(engine.items.len() == 1);
        for_cell(0, 0, 2, 2, &mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_add_item_handle_duplicated_id() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();

        assert!(engine.add_item("0".to_string(), 0, 0, 2, 2).is_err())
    }

    #[test]
    fn test_add_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        let item_1_id = engine.add_item("1".to_string(), 0, 0, 2, 2).unwrap();

        // Item 0 should stay in position 0, 0
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 2);
        item_0
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
                Ok(())
            })
            .unwrap();

        // Item 1 should go to position 0, 2
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 0);
        item_1
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_1_id);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_remove_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 3).unwrap();
        engine.remove_item(&item_0_id).unwrap();
        for_cell(0, 0, 2, 3, &mut |x, y| {
            let value = engine.grid.get(y, x).unwrap();
            assert_eq!(value, &None);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_move_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        engine.move_item(&item_0_id, 1, 1).unwrap();

        // Asserts that its present on the new position
        for_cell(1, 1, 2, 2, &mut |x, y| {
            let item_on_expected_position = engine.grid.get(y, x).unwrap().as_ref().unwrap();
            assert_eq!(item_on_expected_position, &item_0_id);
            Ok(())
        })
        .unwrap();

        // Asserts that its not present on the old position
        for_cell(0, 0, 1, 1, &mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap(), &None);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_move_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        let item_1_id = engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();
        engine.move_item("0", 0, 1).unwrap();

        // Item 0 should go to position 0, 1
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 1);
        item_0
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
                Ok(())
            })
            .unwrap();

        // Item 1 should go to position 0, 3
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 3);
        item_1
            .for_cell(&mut |x, y| {
                assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_1_id);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_will_collides_with() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 1, 2).unwrap();

        // Asserts that does not collide with self
        assert_eq!(
            engine.will_collides_with(
                &engine.items.get(&item_0_id).unwrap(),
                0,
                0,
                &engine.grid.clone()
            ),
            Vec::<String>::new()
        );

        // Asserts that does not collide with empty position
        assert_eq!(
            engine.will_collides_with(
                &engine.items.get(&item_0_id).unwrap(),
                2,
                2,
                &engine.grid.clone()
            ),
            Vec::<String>::new()
        );

        // Asserts that collide with occupied position
        let item_1_id = engine.add_item("1".to_string(), 1, 2, 1, 2).unwrap();

        // Full collision
        assert_eq!(
            engine.will_collides_with(
                &engine.items.get(&item_0_id).unwrap(),
                1,
                2,
                &engine.grid.clone()
            ),
            vec![item_1_id.clone()]
        );

        // Partial collision
        assert_eq!(
            engine.will_collides_with(
                &engine.items.get(&item_0_id).unwrap(),
                1,
                1,
                &engine.grid.clone()
            ),
            vec![item_1_id.clone()]
        );
    }

    #[test]
    fn test_get_nodes() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        let item_1_id = engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();

        let nodes = engine.get_grid_view().get_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, item_0_id);
        assert_eq!(nodes[1].id, item_1_id);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();

        let serialized = engine.get_grid_view().serialized_as_str();
        let deserialized_engine = GridEngine::from_str(&serialized).unwrap();

        assert_eq!(
            engine.get_grid_view().get_nodes(),
            deserialized_engine.get_grid_view().get_nodes()
        );
        assert_eq!(
            engine.get_grid_view().get_grid_formatted(),
            deserialized_engine.get_grid_view().get_grid_formatted()
        );
    }
}
