use crate::{engine_events::EventListener, error::GridError};
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

// TODO, remove unnecessary clones
// TODO, Handle all `UnhandledError``

/// Executes the given callback function for each cell within the specified rectangular region.
///
/// # Arguments
///
/// * `x` - The starting x-coordinate of the region.
/// * `y` - The starting y-coordinate of the region.
/// * `w` - The width of the region.
/// * `h` - The height of the region.
/// * `callback` - A mutable reference to a function that takes the x and y coordinates of each cell as arguments.
/// ```
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

/// Represents a node in the grid with its position and size.
///
/// # Fields
///
/// * `id` - The unique identifier of the node.
/// * `x` - The x-coordinate of the top-left corner of the node.
/// * `y` - The y-coordinate of the top-left corner of the node.
/// * `w` - The width of the node.
/// * `h` - The height of the node.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Node {
    pub id: String,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Node {
    fn new(id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        Node { id, x, y, w, h }
    }

    /// Executes the specified callback function for each cell within the node's boundaries.
    ///
    /// # Arguments
    ///
    /// * `callback` - A mutable reference to a function that takes the x and y coordinates of a cell as arguments.
    fn for_cell(
        &self,
        callback: &mut impl FnMut(usize, usize) -> Result<(), GridError>,
    ) -> Result<(), GridError> {
        for_cell(self.x, self.y, self.w, self.h, callback)
    }
}

/// Represents the data for an add change operation.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct AddChangeData {
    value: Node,
}

/// Represents the data for an remove change operation
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct RemoveChangeData {
    value: Node,
}

/// Represents the data for an move change operation
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct MoveChangeData {
    old_value: Node,
    new_value: Node,
}

/// Change operation types
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
enum ChangeType {
    Add(AddChangeData),
    Remove(RemoveChangeData),
    Move(MoveChangeData),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Change {
    change_type: ChangeType,
}

impl Change {
    fn new(change_type: ChangeType) -> Change {
        Change { change_type }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EventValue {
    Change(Change),
    BatchChange(Vec<Change>),
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum EventName {
    Change,
    BatchChange,
}

/// Represents a grid engine that manages a grid of nodes with their positions and sizes.
///
/// The `GridEngine` struct provides methods for adding, removing, and moving nodes within the grid.
/// It handles collisions between nodes and ensures that they are placed correctly within the grid.
///
/// # Fields
///
/// * `grid` - The grid that holds the nodes.
/// * `items` - A hashmap that maps node IDs to their corresponding nodes.
///
/// # Examples
///
/// ```
/// use grid_engine::engine::GridEngine;
/// let mut engine = GridEngine::new(10, 10);
///
/// // Add a node to the grid
/// let item_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
///
/// // Move the node to a new position
/// engine.move_item(&item_id, 1, 1);
///
/// // Remove the node from the grid
/// engine.remove_item(&item_id);
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct GridEngine {
    grid: Grid<Option<String>>,
    items: HashMap<String, Node>,
    pending_changes: Vec<Change>,
    #[serde(skip)]
    pub events: EventListener<EventName, EventValue>,
}

impl GridEngine {
    /// Creates a new `GridEngine` instance with the specified number of rows and columns.
    ///
    /// # Arguments
    ///
    /// * `rows` - The number of rows in the grid.
    /// * `cols` - The number of columns in the grid.
    ///
    /// # Returns
    ///
    /// A new `GridEngine` instance.
    // #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> GridEngine {
        GridEngine {
            grid: Grid::new(rows, cols),
            items: HashMap::new(),
            pending_changes: Vec::new(),
            events: EventListener::default(),
        }
    }

    pub fn from_str(serialized: &str) -> Result<GridEngine, GridError> {
        match serde_json::from_str(serialized) {
            Ok(engine) => Ok(engine),
            Err(err) => {
                println!("Error deserializing GridEngine {:?}", err);
                Err(GridError::new("Error deserializing GridEngine"))
            }
        }
    }

    /// Creates a new `Node` and adds it to the grid.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the node.
    /// * `y` - The y-coordinate of the top-left corner of the node.
    /// * `w` - The width of the node.
    /// * `h` - The height of the node.
    ///
    /// # Returns
    ///
    /// The newly created `Node`.
    fn new_node(&mut self, id: String, x: usize, y: usize, w: usize, h: usize) -> Node {
        let node = Node::new(id, x, y, w, h);
        node
    }

    fn create_add_change(&mut self, node: &Node) {
        self.pending_changes
            .push(Change::new(ChangeType::Add(AddChangeData {
                value: node.clone(),
            })));
    }

    /// Adds an item to the grid at the specified position.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the item.
    /// * `y` - The y-coordinate of the top-left corner of the item.
    /// * `w` - The width of the item.
    /// * `h` - The height of the item.
    ///
    /// # Returns
    ///
    /// The ID of the added item.
    pub fn add_item(
        &mut self,
        id: String,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Result<String, GridError> {
        if self.items.get(&id).is_some() {
            return Err(GridError::new("Id already exists"));
        };

        let node = self.new_node(id, x, y, w, h);
        let node_id = node.id.to_string();

        self.handle_collision(&node, x, y);

        self.create_add_change(&node);

        self.apply_changes();

        Ok(node_id)
    }

    fn create_remove_change(&mut self, node: &Node) {
        self.pending_changes
            .push(Change::new(ChangeType::Remove(RemoveChangeData {
                value: node.clone(),
            })));
    }

    /// Removes an item from the grid.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the item to remove.
    pub fn remove_item(&mut self, id: &str) -> Result<(), GridError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridError::new("Item not found"))?,
        }
        .clone();

        self.create_remove_change(&node);

        self.apply_changes();
        Ok(())
    }

    /// Handles collisions between nodes and moves them if necessary.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check for collisions.
    /// * `x` - The x-coordinate of the top-left corner of the node.
    /// * `y` - The y-coordinate of the top-left corner of the node.
    fn handle_collision(&mut self, node: &Node, x: usize, y: usize) {
        let collides_with = self.will_collides_with(node, x, y);
        if collides_with.len() > 0 {
            for collided_id in collides_with {
                let collided = self.items.get(&collided_id).unwrap().clone();
                self.create_move_change(&collided, collided.x, y + node.h);
            }
        }
    }

    fn create_move_change(&mut self, node: &Node, new_x: usize, new_y: usize) {
        let old_node = node.clone();

        self.handle_collision(node, new_x, new_y);

        self.pending_changes
            .push(Change::new(ChangeType::Move(MoveChangeData {
                old_value: old_node,
                new_value: Node::new(node.id.to_string(), new_x, new_y, node.w, node.h),
            })));
    }

    /// Moves an item to a new position in the grid.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the item to move.
    /// * `new_x` - The new x-coordinate of the top-left corner of the item.
    /// * `new_y` - The new y-coordinate of the top-left corner of the item.
    pub fn move_item(&mut self, id: &str, new_x: usize, new_y: usize) -> Result<(), GridError> {
        let node = match self.items.get(id) {
            Some(node) => node,
            None => Err(GridError::new("Item not found"))?,
        };

        self.create_move_change(&node.clone(), new_x, new_y);

        self.apply_changes();

        Ok(())
    }

    fn apply_changes(&mut self) {
        // Needs to think if the events should be triggered before or after the changes
        self.events.trigger_event(
            EventName::BatchChange,
            EventValue::BatchChange(
                self.pending_changes
                    .iter()
                    .map(|change| change.clone())
                    .collect(),
            ),
        );

        for change in self.pending_changes.iter() {
            self.events
                .trigger_event(EventName::Change, EventValue::Change(change.clone()));

            match &change.change_type {
                ChangeType::Add(data) => {
                    let node = &data.value;

                    node.for_cell(&mut |x, y| {
                        let element_at_position = self.grid.get_mut(y, x);

                        match element_at_position {
                            Some(cell) => {
                                *cell = Some(node.id.to_string());
                                Ok(())
                            }
                            None => Err(GridError::new("Error adding item to grid")),
                        }
                    })
                    .expect("UnhandledError");

                    self.items.insert(node.id.to_string(), node.clone());
                }
                ChangeType::Remove(data) => {
                    let node = &data.value;

                    node.for_cell(&mut |x, y| {
                        let element_at_position = self.grid.get_mut(y, x);

                        match element_at_position {
                            Some(cell) => {
                                *cell = None;
                                Ok(())
                            }
                            None => Err(GridError::new("Error removing item from grid")),
                        }
                    })
                    .expect("UnhandledError");

                    self.items.remove(&node.id);
                }
                ChangeType::Move(data) => {
                    let node = &data.new_value;
                    let old_node = &data.old_value;

                    old_node
                        .for_cell(&mut |x, y| {
                            let element_at_position = self.grid.get_mut(y, x);

                            match element_at_position {
                                Some(cell) => {
                                    *cell = None;
                                    Ok(())
                                }
                                None => Err(GridError::new("Error moving item from grid")),
                            }
                        })
                        .expect("UnhandledError");

                    self.items.insert(node.id.to_string(), node.clone());
                    node.for_cell(&mut |x, y| {
                        let element_at_position = self.grid.get_mut(y, x);

                        match element_at_position {
                            Some(cell) => {
                                *cell = Some(node.id.to_string());
                                Ok(())
                            }
                            None => Err(GridError::new("Error moving item to grid")),
                        }
                    })
                    .expect("UnhandledError");
                }
            }
        }
        // Needs to think if could exists a problem of possibly not processed changes being also removed.
        self.pending_changes.clear();
    }

    /// Checks if a node will collide with any other nodes at the specified position.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check for collisions.
    /// * `x` - The x-coordinate of the top-left corner of the node.
    /// * `y` - The y-coordinate of the top-left corner of the node.
    ///
    /// # Returns
    ///
    /// A vector containing the IDs of the nodes that the specified node will collide with.
    fn will_collides_with(&self, node: &Node, x: usize, y: usize) -> Vec<String> {
        let mut collides_with = Vec::new();
        for_cell(x, y, node.w, node.h, &mut |x, y| {
            if let Some(cell) = self.grid.get(y, x) {
                if cell.is_some() && cell.as_ref().unwrap() != &node.id {
                    if !collides_with.contains(&cell.as_ref().unwrap().to_string()) {
                        collides_with.push(cell.as_ref().unwrap().to_string());
                    }
                }
            }
            Ok(())
        })
        .expect("UnhandledError");

        collides_with
    }

    /// Get the nodes sorted by id
    pub fn get_nodes(&self) -> Vec<Node> {
        let mut cloned: Vec<Node> = self.items.values().cloned().collect();
        // Would be better to sort by some created_at
        cloned.sort_by_key(|n| n.id.clone());
        cloned
    }

    /// Prints answer of get_grid_formatted
    pub fn print_grid(&self) {
        println!("{}", self.get_grid_formatted());
    }

    /// Format grid nodes to string
    pub fn get_grid_formatted(&self) -> String {
        let mut grid_str = String::new();
        self.grid.iter_rows().for_each(|row| {
            row.for_each(|cell| match cell {
                Some(item) => {
                    grid_str.push_str(&format!("[{}]", item));
                }
                None => {
                    grid_str.push_str("[ ]");
                }
            });
            grid_str.push_str("\n");
        });

        grid_str
    }

    pub fn serialized_as_str(&self) -> String {
        serde_json::to_string(self).unwrap()
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
            engine.will_collides_with(&engine.items.get(&item_0_id).unwrap(), 0, 0),
            Vec::<String>::new()
        );

        // Asserts that does not collide with empty position
        assert_eq!(
            engine.will_collides_with(&engine.items.get(&item_0_id).unwrap(), 2, 2),
            Vec::<String>::new()
        );

        // Asserts that collide with occupied position
        let item_1_id = engine.add_item("1".to_string(), 1, 2, 1, 2).unwrap();

        // Full collision
        assert_eq!(
            engine.will_collides_with(&engine.items.get(&item_0_id).unwrap(), 1, 2),
            vec![item_1_id.clone()]
        );

        // Partial collision
        assert_eq!(
            engine.will_collides_with(&engine.items.get(&item_0_id).unwrap(), 1, 1),
            vec![item_1_id.clone()]
        );
    }

    #[test]
    fn test_get_nodes() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        let item_1_id = engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();

        let nodes = engine.get_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, item_0_id);
        assert_eq!(nodes[1].id, item_1_id);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let mut engine = GridEngine::new(10, 10);
        engine.add_item("0".to_string(), 0, 0, 2, 2).unwrap();
        engine.add_item("1".to_string(), 0, 2, 2, 2).unwrap();

        let serialized = engine.serialized_as_str();
        let deserialized_engine = GridEngine::from_str(&serialized).unwrap();

        assert_eq!(engine.get_nodes(), deserialized_engine.get_nodes());
        assert_eq!(
            engine.get_grid_formatted(),
            deserialized_engine.get_grid_formatted()
        );
    }
}
