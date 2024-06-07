use grid::Grid;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

/// Executes the given callback function for each cell within the specified rectangular region.
///
/// # Arguments
///
/// * `x` - The starting x-coordinate of the region.
/// * `y` - The starting y-coordinate of the region.
/// * `w` - The width of the region.
/// * `h` - The height of the region.
/// * `callback` - A mutable reference to a function that takes the x and y coordinates of each cell as arguments.
///
/// # Example
///
/// ```
/// fn print_coordinates(x: usize, y: usize) {
///     println!("Cell coordinates: ({}, {})", x, y);
/// }
///
/// for_cell(0, 0, 3, 3, &mut print_coordinates);
/// // Output:
/// // Cell coordinates: (0, 0)
/// // Cell coordinates: (0, 1)
/// // Cell coordinates: (0, 2)
/// // Cell coordinates: (1, 0)
/// // Cell coordinates: (1, 1)
/// // Cell coordinates: (1, 2)
/// // Cell coordinates: (2, 0)
/// // Cell coordinates: (2, 1)
/// // Cell coordinates: (2, 2)
/// ```
fn for_cell(x: usize, y: usize, w: usize, h: usize, callback: &mut impl FnMut(usize, usize)) {
    for x in x..x + w {
        for y in y..y + h {
            callback(x, y);
        }
    }
}

/// Represents a node in the grid with its position, size, and content.
///
/// # Type Parameters
///
/// * `GridContent` - The type of content stored in the node.
///
/// # Fields
///
/// * `id` - The unique identifier of the node.
/// * `x` - The x-coordinate of the top-left corner of the node.
/// * `y` - The y-coordinate of the top-left corner of the node.
/// * `w` - The width of the node.
/// * `h` - The height of the node.
/// * `content` - The content stored in the node.
///
/// # Examples
///
/// ```
/// let node = Node::new("1".to_string(), 0, 0, 2, 2, "Content");
/// assert_eq!(node.id, "1");
/// assert_eq!(node.x, 0);
/// assert_eq!(node.y, 0);
/// assert_eq!(node.w, 2);
/// assert_eq!(node.h, 2);
/// assert_eq!(node.content, "Content");
/// ```
#[derive(Clone, Debug)]
pub struct Node<GridContent> {
    id: String,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    content: GridContent,
}

impl<GridContent> Node<GridContent> {
    fn new(
        id: String,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        content: GridContent,
    ) -> Node<GridContent> {
        Node {
            id,
            x,
            y,
            w,
            h,
            content,
        }
    }

    /// Executes the specified callback function for each cell within the node's boundaries.
    ///
    /// # Arguments
    ///
    /// * `callback` - A mutable reference to a function that takes the x and y coordinates of a cell as arguments.
    fn for_cell(&self, callback: &mut impl FnMut(usize, usize)) {
        for_cell(self.x, self.y, self.w, self.h, callback);
    }
}


/// Represents a grid engine that manages a grid of nodes with their positions, sizes, and content.
///
/// The `GridEngine` struct provides methods for adding, removing, and moving nodes within the grid.
/// It handles collisions between nodes and ensures that they are placed correctly within the grid.
///
/// # Type Parameters
///
/// * `GridContent` - The type of content stored in the nodes.
///
/// # Fields
///
/// * `grid` - The grid that holds the nodes.
/// * `items` - A hashmap that maps node IDs to their corresponding nodes.
/// * `new_id` - The ID to be assigned to the next node added to the grid.
///
/// # Examples
///
/// ```
/// let mut engine = GridEngine::new(10, 10);
///
/// // Add a node to the grid
/// let item_id = engine.add_item(0, 0, 2, 2, "Content");
///
/// // Move the node to a new position
/// engine.move_item(&item_id, 1, 1);
///
/// // Remove the node from the grid
/// engine.remove_item(&item_id);
/// ```
#[derive(Debug)]
pub struct GridEngine<GridContent: Default + Display + Clone> {
    grid: Grid<Option<String>>,
    items: HashMap<String, Node<GridContent>>,
    new_id: u32,
}

impl<G: Default + Display + Clone + Debug> GridEngine<G> {
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
    pub fn new(rows: usize, cols: usize) -> GridEngine<G> {
        GridEngine {
            grid: Grid::new(rows, cols),
            items: HashMap::new(),
            new_id: 0,
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
    /// * `content` - The content of the node.
    ///
    /// # Returns
    ///
    /// The newly created `Node`.
    pub fn new_node(&mut self, x: usize, y: usize, w: usize, h: usize, content: G) -> Node<G> {
        let node = Node::new(self.new_id.to_string(), x, y, w, h, content);
        self.new_id += 1;
        node
    }

    /// Adds an item to the grid at the specified position.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the item.
    /// * `y` - The y-coordinate of the top-left corner of the item.
    /// * `w` - The width of the item.
    /// * `h` - The height of the item.
    /// * `content` - The content of the item.
    ///
    /// # Returns
    ///
    /// The ID of the added item.
    pub fn add_item(&mut self, x: usize, y: usize, w: usize, h: usize, content: G) -> String {
        let node = self.new_node(x, y, w, h, content);

        self.handle_collision(&node, x, y);

        self.items.insert(node.id.to_string(), node.clone());
        let grid = &mut self.grid;
        node.for_cell(&mut |x, y| {
            let element_at_position = grid.get_mut(y, x);

            match element_at_position {
                Some(cell) => {
                    *cell = Some(node.id.to_string());
                }
                None => {
                    println!("Error adding item to grid");
                }
            }
        });

        node.id.to_string()
    }

    /// Removes an item from the grid.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the item to remove.
    pub fn remove_item(&mut self, id: &str) {
        if let Some(node) = self.items.get(id) {
            node.for_cell(&mut |x, y| {
                let element_at_position = self.grid.get_mut(y, x);

                match element_at_position {
                    Some(cell) => {
                        *cell = None;
                    }
                    None => {
                        println!("Error removing item from grid");
                    }
                }
            });
        }

        self.items.remove(id);
    }

    /// Handles collisions between nodes and moves them if necessary.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check for collisions.
    /// * `x` - The x-coordinate of the top-left corner of the node.
    /// * `y` - The y-coordinate of the top-left corner of the node.
    fn handle_collision(&mut self, node: &Node<G>, x: usize, y: usize) {
        let collides_with = self.will_collides_with(node, x, y);
        if collides_with.len() > 0 {
            for collided_id in collides_with {
                let collided = self.items.get(&collided_id).unwrap();
                self.move_item(&collided_id, collided.x, y + node.h);
            }
        }
    }

    /// Moves an item to a new position in the grid.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the item to move.
    /// * `new_x` - The new x-coordinate of the top-left corner of the item.
    /// * `new_y` - The new y-coordinate of the top-left corner of the item.
    pub fn move_item(&mut self, id: &str, new_x: usize, new_y: usize) {
        if let Some(node) = self.items.get(id) {
            self.handle_collision(&node.clone(), new_x, new_y);

            let node = self.items.get_mut(id).unwrap();
            node.for_cell(&mut |old_x, old_y| {
                let element_at_position = self.grid.get_mut(old_y, old_x);

                match element_at_position {
                    Some(cell) => {
                        *cell = None;
                    }
                    None => {
                        println!("Error moving item from grid");
                    }
                }
            });

            node.x = new_x;
            node.y = new_y;

            node.for_cell(&mut |x, y| {
                let element_at_position = self.grid.get_mut(y, x);

                match element_at_position {
                    Some(cell) => {
                        *cell = Some(id.to_string());
                    }
                    None => {
                        println!("Error moving item to grid");
                    }
                }
            });
        }
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
    pub fn will_collides_with(&self, node: &Node<G>, x: usize, y: usize) -> Vec<String> {
        let mut collides_with = Vec::new();
        for_cell(x, y, node.w, node.h, &mut |x, y| {
            if let Some(cell) = self.grid.get(y, x) {
                if cell.is_some() && cell.as_ref().unwrap() != &node.id {
                    if !collides_with.contains(&cell.as_ref().unwrap().to_string()) {
                        collides_with.push(cell.as_ref().unwrap().to_string());
                    }
                }
            }
        });

        collides_with
    }

    /// Prints the grid to the console.
    pub fn print_grid(&self) {
        self.grid.iter_rows().for_each(|row| {
            row.for_each(|cell| match cell {
                Some(item) => {
                    print!("[{}]", item);
                }
                None => {
                    print!("[ ]");
                }
            });
            println!();
        });
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_for_cell() {
        let mut results = Vec::new();
        let mut callback = |x: usize, y: usize| {
            results.push((x, y));
        };

        for_cell(1, 2, 2, 2, &mut callback);

        assert_eq!(results, vec![(1, 2), (1, 3), (2, 2), (2, 3)]);
    }

    #[test]
    fn test_add_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 2, 2, 9);

        assert!(engine.items.len() == 1);
        for_cell(0, 0, 2, 2, &mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
        });
    }

    #[test]
    fn test_add_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 2, 2, 1);
        let item_1_id = engine.add_item(0, 0, 2, 2, 2);

        // Item 0 should stay in position 0, 0
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 2);
        item_0.for_cell(&mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
        });

        // Item 1 should go to position 0, 2
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 0);
        item_1.for_cell(&mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_1_id);
        });
    }

    #[test]
    fn test_remove_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 2, 3, 9);
        engine.remove_item(&item_0_id);
        for_cell(0, 0, 2, 3, &mut |x, y| {
            let value = engine.grid.get(y, x).unwrap();
            assert_eq!(value, &None);
        });
    }

    #[test]
    fn test_move_item() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 2, 2, 9);
        engine.move_item(&item_0_id, 1, 1);

        // Asserts that its present on the new position
        for_cell(1, 1, 2, 2, &mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
        });

        // Asserts that its not present on the old position
        for_cell(0, 0, 1, 1, &mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap(), &None);
        });
    }

    #[test]
    fn test_move_item_handle_collision() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 2, 2, 1);
        let item_1_id = engine.add_item(0, 2, 2, 2, 2);
        engine.move_item("0", 0, 1);

        // Item 0 should go to position 0, 1
        let item_0 = engine.items.get(&item_0_id).unwrap();
        assert_eq!(item_0.x, 0);
        assert_eq!(item_0.y, 1);
        item_0.for_cell(&mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_0_id);
        });

        // Item 1 should go to position 0, 3
        let item_1 = engine.items.get(&item_1_id).unwrap();
        assert_eq!(item_1.x, 0);
        assert_eq!(item_1.y, 3);
        item_1.for_cell(&mut |x, y| {
            assert_eq!(engine.grid.get(y, x).unwrap().as_ref().unwrap(), &item_1_id);
        });
    }

    #[test]
    fn test_will_collides_with() {
        let mut engine = GridEngine::new(10, 10);
        let item_0_id = engine.add_item(0, 0, 1, 2, 9);

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
        let item_1_id = engine.add_item(1, 2, 1, 2, 5);

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
}
