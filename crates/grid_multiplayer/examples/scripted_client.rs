use std::{thread, time};

use grid_engine::grid_engine::{EventName, GridEngine};

enum Interaction {
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

fn print_grid(grid: &mut GridEngine) {
    // print!("\x1B[2J\x1B[1;1H");
    println!("Printing the grid");
    grid.get_grid_view().print_grid();
}

fn handle_interaction(grid: &mut GridEngine, interaction: Interaction) {
    match interaction {
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
    print_grid(grid);
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

async fn scripted_mode() {
    println!("Grid App");

    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:8080/".to_string());

    // Generate random client id using system time
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let client_id = since_the_epoch.as_secs() ^ since_the_epoch.subsec_nanos() as u64;

    let grid_multiplayer = GridMultiplayerClient::initialize(url, client_id.to_string()).await.unwrap();

    let instructions = vec![
        "add a 2 2 2 4 1",
        "add b 4 2 2 4 2",
        "add c 0 2 2 2",
        "rm b",
        "add d 4 2 2 3 0",
        "add e 2 2 2 4 1",
        "add f 2 2 2 4 1",
        "rm f",
        "add g 2 2 2 4 1",
        "rm a",
        "mv c 1 0",
        "mv c 2 0",
        "mv c 2 2",
        "mv c 3 2",
        "mv c 4 10",
        "mv c 4 6",
        // "mv d 1 1",
        // "mv c 4 6", // Bug
    ];

    for instruction in instructions {
        {
            let mut grid_locked = grid_multiplayer.grid_arc.lock().unwrap();
            handle_interaction(&mut grid_locked, Interaction::from_str(instruction));
        }
        thread::sleep(time::Duration::from_millis(750))
    }
}

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use grid_multiplayer::client::GridMultiplayerClient;

#[tokio::main]
async fn main() {
    // interactive_mode();
    scripted_mode().await;
}
