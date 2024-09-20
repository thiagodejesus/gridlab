use std::collections::HashMap;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use grid::Grid;
use serde::{Deserialize, Serialize};

use crate::grid_engine::{GridEngine, Node};

#[derive(Serialize, Deserialize, Clone)]
pub struct GridView {
    pub(crate) grid: Grid<Option<String>>,
    pub(crate) items: HashMap<String, Node>,
}

impl GridView {
    pub fn new(grid_engine: &GridEngine) -> GridView {
        GridView {
            grid: grid_engine.grid.clone(),
            items: grid_engine.items.clone(),
        }
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
        grid_str.push_str("  ");
        for i in 0..self.grid.cols() {
            grid_str.push_str(&format!(" {} ", i));
        }
        grid_str.push_str("\n");

        self.grid
            .iter_rows()
            .enumerate()
            .for_each(|(row_number, row)| {
                row.enumerate().for_each(|(index, cell)| {
                    if index == 0 {
                        grid_str.push_str(&format!("{:0>2}", row_number));
                    }
                    return match cell {
                        Some(item) => {
                            grid_str.push_str(&format!("[{}]", item));
                        }
                        None => {
                            grid_str.push_str("[ ]");
                        }
                    };
                });
                grid_str.push_str("\n");
            });

        grid_str
    }

    pub fn serialized_as_str(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize GridEngine")
    }

    pub fn hash(&self) -> String {
        // Get the grid serialized as str and return a hash of it
        let serialized = self.serialized_as_str();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        hasher.finish().to_string()
    }
}
