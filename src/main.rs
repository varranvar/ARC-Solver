mod predicate;

use predicate::*;
use ::serde::Deserialize;
use ::serde_json;
use colored::*;
use std::{fs, io, fmt, path::Path};

const TASKS_FILEPATH: &str = "data/training";

fn main() {
    let mut solved = 0;

    for path in fs::read_dir(&Path::new(TASKS_FILEPATH)).unwrap() {
        // Parses data.
        let path_str = format!("{}", path.unwrap().path().display());
        let file = fs::read_to_string(path_str.clone()).expect("Something went wrong reading the file");
        let task: Task = serde_json::from_str(&file[..]).expect("JSON was not well-formatted");

        // Only solves the task if the inputs and outputs have the same sizes.
        if task.train.iter().any(|example| {
            example.input.len() != example.output.len() || example.input[0].len() != example.output[0].len()
        }) {
            continue;
        }
        // Finds solutions.
        let solutions = solve(task.clone());

        // Displays and validates solutions.
        println!("Solutions");
        let mut solutions_valid = true;
        for (generated_solution, real_solution) in solutions.iter().zip(task.test.iter()) {
            println!("{generated_solution}\n");
            solutions_valid &= generated_solution == real_solution;
        }

        if solutions_valid {
            solved += 1;
            println!("{}", "The solutions were correct!".green());
        } else {
            println!("{}", "The solutions were not correct.".red());
        }

        // Waits for user input.
        io::stdin().read_line(&mut String::new()).ok();
    }

    println!("Number of tasks solved: {solved} / 100");
}

type Grid = Vec<Vec<u8>>;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Pair {
    input: Grid,
    output: Grid,
}

#[derive(Debug, Clone, Deserialize)]
struct Task {
    train: Vec<Pair>,
    test: Vec<Pair>,
}

fn solve(mut task: Task) -> Vec<Pair> {
    // recolor
    let ordering = color_ordering(&task.train.first().unwrap());
    for pair in &mut task.train {
        recolor(pair, &ordering);
    }

    // induce
    let rules = induce(&task.train);
    
    //for rule in &rules {
    //    println!("{rule}");
    //}

    // deduce
    task.test
        .into_iter()
            .map(|mut test| {
                test.output = vec![vec![0; test.input[0].len()]; test.input.len()];
                let test_ordering = color_ordering(&test);
                recolor(&mut test, &ordering);
                let mut solution = deduce(test, &rules);
                recolor(&mut solution, &test_ordering);
                solution
            })
        .collect()
}


// Utility

pub fn cell_to_string(cell: u8) -> String {
    match cell {
        0 => "█".black(),
        1 => "█".blue(),
        2 => "█".red(),
        3 => "█".green(),
        4 => "█".yellow(),
        5 => "█".truecolor(170, 170, 170),
        6 => "█".truecolor(240, 18, 190),
        7 => "█".truecolor(255, 133, 27),
        8 => "█".cyan(),
        9 => "█".magenta(),
        10 => "?".truecolor(100, 100, 100),
        _ => "!".bright_red(),
    }
    .to_string()
}

/* 
fn grid_to_string(grid: &Grid) -> String {
    let mut string = String::new();
    for x in 0..grid.len() {
        for y in 0..grid[0].len() {
            string.push_str(&cell_to_string(grid[x][y])[..]);
        }
        string.push_str("\n");
    }
    string
}
*/

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();
        let max_height = std::cmp::max(self.input[0].len(), self.output[0].len());

        for y in 0.. max_height {
            for x in 0..self.input.len() {
                string.push_str(&cell_to_string(self.input[x][y])[..]);
            }

            if y == max_height / 2 {
                string.push_str(" → ");
            } else {
                string.push_str("   ");
            }

            for x in 0..self.output.len() {
                string.push_str(&cell_to_string(self.output[x][y])[..]);
            }
            
            string.push_str("\n");
        }
        
        write!(f, "{string}")
    }
}