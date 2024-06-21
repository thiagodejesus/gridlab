use std::{thread, time};

use grid_engine::grid_engine::GridEngine;

enum Interaction {
    PrintGrid,
    AddItem(String, usize, usize, usize, usize),
    MoveItem(String, usize, usize),
    RemoveItem(String),
    InvalidInteraction(String),
}

impl Interaction {
    fn from_str(input: &str) -> Interaction {
        // Should match input starts with
        let mut parts = input.split_whitespace();
        let action = parts.next().unwrap_or("");

        match action {
            "print" => Interaction::PrintGrid,
            "add" => {
                println!("{}", input);
                let id = parts.next().expect("Expect id").to_string();

                let x = parts
                    .next()
                    .expect("Expect X")
                    .parse()
                    .expect("Expect x to be number");
                let y = parts
                    .next()
                    .expect("Expect Y")
                    .parse()
                    .expect("Expect y to be number");
                let w = parts
                    .next()
                    .expect("Expect W")
                    .parse()
                    .expect("Expect w to be number");
                let h = parts
                    .next()
                    .expect("Expect H")
                    .parse()
                    .expect("Expect h to be number");

                Interaction::AddItem(id, x, y, w, h)
            }
            "rm" => {
                let id = parts.next().expect("Expect ID");
                Interaction::RemoveItem(id.to_string())
            }
            "mv" => {
                let id = parts.next().expect("Expect ID");
                let x = parts
                    .next()
                    .expect("Expect X")
                    .parse()
                    .expect("Expect x to be number");
                let y = parts
                    .next()
                    .expect("Expect Y")
                    .parse()
                    .expect("Expect y to be number");
                Interaction::MoveItem(id.to_string(), x, y)
            }
            _ => Interaction::InvalidInteraction(input.to_string()),
        }
    }
}

#[derive(Clone, Debug)]
struct GridContent {
    name: String,
}

impl Default for GridContent {
    fn default() -> Self {
        GridContent {
            name: "0".to_string(),
        }
    }
}

impl std::fmt::Display for GridContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn handle_interaction(grid: &mut GridEngine, interaction: Interaction) {
    match interaction {
        Interaction::PrintGrid => {
            print!("\x1B[2J\x1B[1;1H");
            println!("Printing the grid");
            grid.print_grid();
        }
        Interaction::AddItem(id, x, y, w, h) => {
            println!("Adding item to the grid");
            grid.add_item(id, x, y, w, h).unwrap();
        }
        Interaction::RemoveItem(id) => {
            println!("Removing item {} from the grid", &id);
            grid.remove_item(&id).unwrap();
        }
        Interaction::MoveItem(id, x, y) => {
            println!("Moving item {} to ({}, {})", &id, x, y);
            grid.move_item(&id, x, y).unwrap();
        }
        Interaction::InvalidInteraction(instruction) => {
            println!("Invalid interaction: {}", instruction);
        }
    }
}

// fn interactive_mode() {
//     println!("Grid App");

//     let mut grid = GridEngine::new(8, 12);

//     loop {
//         // Reads some input from the user and prints it back
//         let mut input = String::new();
//         std::io::stdin().read_line(&mut input).unwrap();

//         let input = input.trim();

//         handle_interaction(&mut grid, Interaction::from_str(input));
//     }
// }

fn scripted_mode() {
    println!("Grid App");

    let mut grid = GridEngine::new(16, 12);

    let instructions = vec![
        //   x y w h itemContent
        "add a 2 2 2 4 1",
        "print",
        "add b 4 2 2 4 2",
        "print",
        "add c 0 2 2 2",
        "print",
        //  id
        "rm b",
        "print",
        "add d 4 2 2 3 0",
        "print",
        "add e 2 2 2 4 1",
        "print",
        "add f 2 2 2 4 1",
        "print",
        "rm f",
        "print",
        "add g 2 2 2 4 1",
        "print",
        "rm a",
        "print",
        // id x y
        "mv c 1 0",
        "print",
        "mv c 2 0",
        "print",
        "mv c 2 2",
        "print",
        "mv c 3 2",
        "print",
    ];

    // // For each add instruction create an id
    // let mut ids: Vec<String> = vec![];
    // instructions.iter().for_each(|instruction| {
    //     let interaction = Interaction::from_str(instruction);
    //     match interaction {
    //         Interaction::AddItem(_, _, _, _, item) => {
    //             ids.push(item);
    //         }
    //         _ => {}
    //     }
    // });

    for instruction in instructions {
        handle_interaction(&mut grid, Interaction::from_str(instruction));
        thread::sleep(time::Duration::from_millis(200))
    }
}

fn main() {
    // interactive_mode();
    scripted_mode();
}
