use crate::*;
use std::{
    collections::{HashSet, VecDeque},
    fmt
};

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North = 0,
    West = 1,
    South = 2,
    East = 3,
    Above = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule(Direction, u8, u8);

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?} of {}", 
            cell_to_string(self.1), 
            self.0, 
            cell_to_string(self.2)
        )
    }
}

type Rules = HashSet<Rule>;

pub fn induce(examples: &Vec<Pair>) -> Rules {
    examples
        .iter()
        // Generates rules for each example.
        .map(|example| {
            let mut rules = HashSet::new();
            let width = example.input.len();
            let height = example.input[0].len();

            for x in 0..width {
                for y in 0..height {
                    if x > 0 {
                        rules.insert(Rule(
                            Direction::West,
                            example.output[x][y],
                            example.output[x - 1][y],
                        ));
                    }

                    if x < width - 1 {
                        rules.insert(Rule(
                            Direction::East,
                            example.output[x][y],
                            example.output[x + 1][y],
                        ));
                    }

                    if y > 0 {
                        rules.insert(Rule(
                            Direction::North,
                            example.output[x][y],
                            example.output[x][y - 1],
                        ));
                    }

                    if y < height - 1 {
                        rules.insert(Rule(
                            Direction::South,
                            example.output[x][y],
                            example.output[x][y + 1],
                        ));
                    }

                    rules.insert(Rule(
                        Direction::Above,
                        example.output[x][y],
                        example.input[x][y],
                    ));
                }
            }

            rules
        })
        // Filters out non-constant rules.
        .reduce(|left_rules, right_rules| left_rules.union(&right_rules).cloned().collect())
        .unwrap()
}

type Domain = [bool; 10];
type Domains = Vec<Vec<Domain>>;

pub fn deduce(mut test: Pair, rules: &Rules) -> Pair {
    let width = test.input.len();
    let height = test.input[0].len();
    let mut domains = vec![vec![[true; 10]; height]; width];

    // Applies constraints from the input frame.
    for x in 0..width {
        for y in 0..height {
            let mut domain = [false; 10];
            
            let input_color = test.input[x][y];
            for rule in rules {
                if rule.0 == Direction::Above && rule.2 == input_color {
                    domain[rule.1 as usize] = true;
                }
            }

            domains[x][y] = domain;
        }
    }

    // Propogates changes for freshly initialized output domains.
    for x in 0..width {
        for y in 0..height {
            if !propogate_changes(rules, &mut domains, x, y) {
                //println!("Error progating constraints from the input.");
                //return test;
            }
        }
    }

    // Finds solution.
    let mut iterations = 0;
    let solution = solve(rules, domains, &mut iterations, 0).unwrap_or(vec![vec![[false; 10]; height]; width]);

    // Converts solution domains into a grid.
    let mut uncollapsed_tile_count = 0;
    let mut no_solution_count = 0;

    for x in 0..width {
        for y in 0..height {
            let domain = solution[x][y];
            let support_count = domain.into_iter().filter(|support| *support).count();
            if support_count == 0 {
                no_solution_count += 1;
            } else if support_count > 1 {
                uncollapsed_tile_count += 1;
            } else {
                for (color, support) in solution[x][y].iter().enumerate() {
                    if *support {
                        test.output[x][y] = color as u8;
                        break;
                    }
                }
            }
            
        }
    }

    println!("Uncollapsed tile count: {uncollapsed_tile_count}");
    println!("No solution count: {no_solution_count}");

    test
}

const MAX_ITERATION_COUNT: usize = 500;

// Searches for a valid solution.
fn solve(
    rules: &Rules,
    domains: Domains,
    iterations: &mut usize,
    depth: usize,
) -> Option<Domains> {
    // Returns if the number of iterations exceeds the maximum iteration count.
    *iterations += 1;
    if *iterations > MAX_ITERATION_COUNT {
        return Some(domains);
    }

    // Finds unsolved tiles and their entropies.
    let mut unsolved_tiles = domains
        .iter()
        .enumerate()
        .map(|(x, column)| {
            column.iter().enumerate().map(move |(y, domain)| {
                (
                    x,
                    y,
                    domain
                        .iter()
                        .filter(|support| **support)
                        .count(),
                )
            })
        })
        .flatten()
        // Removes tiles that have already been solved or have too much entropy.
        .filter(|(_, _, entropy)| *entropy > 1)
        .collect::<Vec<(usize, usize, usize)>>();

    // Returns if there are no more unsolved tiles.
    if unsolved_tiles.len() == 0 {
        return Some(domains);
    }

    // Sorts the unsolved tiles by least entropy.
    unsolved_tiles.sort_unstable_by_key(|(_, _, entropy)| *entropy);
    let unsolved_tile_count = unsolved_tiles.len();

    //println!("Iteration {iterations}, Depth {depth}, {} left", unsolved_tile_count);
    //println!("{unsolved_tiles:?}");


    // For every undecided tile:
    for (i, (x, y, _)) in unsolved_tiles.into_iter().enumerate() {
        //println!("Depth {} progress: {} / {} tiles checked", depth, i, unsolved_tile_count);

        //print!(" {x},{y} ");
        // For every open color:
        for (color, support) in domains[x][y].iter().enumerate() {
           // print!("{color}-");
            // If the color is not eliminated:
            if *support {
                // Collapses the domain.
                let mut new_domains = domains.clone();
                new_domains[x][y] = [false; 10];
                new_domains[x][y][color] = true;
                
                // Propogates changes and skips if there was a failure.
                if propogate_changes(rules, &mut new_domains, x, y) {
                    // Solves the next tile.
                    if let Some(solution) = solve(rules, new_domains, iterations, depth + 1) {
                        return Some(solution);
                    }
                }
            } 
        }
    }

    None
}


fn propogate_changes(
    rules: &Rules,
    domains: &mut Domains,
    initial_x: usize,
    initial_y: usize,
) -> bool {
    // Propogates changes.
    let width = domains.len();
    let height = domains[0].len();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    queue.push_back((initial_x, initial_y));

    loop {
        if let Some((x, y)) = queue.pop_front() {
            // Propogate changes to neighbors
            for x_offset in [-1, 1].into_iter() {
                for y_offset in [-1, 1].into_iter() {
                    let xx = (x as i32 + x_offset) as usize;
                    let yy = (y as i32 + y_offset) as usize;

                    if xx > 0 && xx < width - 1 && yy > 0 && yy < height - 1 {
                        match constrain_tile(&rules, domains, xx, yy) {
                            ConstraintResult::Collapsed => queue.push_back((xx as usize, yy as usize)),
                            ConstraintResult::Invalid => return false,
                            ConstraintResult::Valid => ()
                        }
                    }
                }
            }
        } else {
            break;
        }
    }

    true
}

// Constrains the domain of a tile based on the value of its neighboring tile
fn constrain_tile(
    rules: &Rules,
    domains: &mut Vec<Vec<[bool; 10]>>,
    x: usize,
    y: usize
) -> ConstraintResult {
    let mut change = false;
    let width = domains.len();
    let height = domains[0].len();

    for (color_index, old_support) in domains[x][y].clone().into_iter().enumerate() {
        if old_support {
            let color = color_index as u8;

            let north_support = y > 0 && supported_by_other(rules, Direction::North, color, &domains[x][y - 1]);
            let west_support = x > 0 && supported_by_other(rules, Direction::West, color, &domains[x - 1][y]);
            let south_support = y < height - 1 && supported_by_other(rules, Direction::South, color, &domains[x][y + 1]);
            let east_support = x < width - 1 && supported_by_other(rules, Direction::East, color, &domains[x + 1][y]);

            let new_support = north_support && west_support && south_support && east_support;
            domains[x][y][color_index] = new_support;

            if !new_support {
                change = true;
            }
        }
    }

    if change {
        let domain_len = domains[x][y]
            .iter()
            .filter(|support| **support)
            .count();
        
        if domain_len == 0 {
            ConstraintResult::Invalid
        } else if domain_len == 1 {
            ConstraintResult::Collapsed
        } else {
            ConstraintResult::Valid
        }
    } else {
        ConstraintResult::Valid
    }

}

enum ConstraintResult {
    Collapsed,
    Valid,
    Invalid,
}

fn supported_by_other(rules: &Rules, direction: Direction, color: u8, other_tile: &Domain) -> bool {
    other_tile
        .iter()
        .enumerate()
        .any(|(other_color, support)|
            if *support {
                rules
                    .iter()
                    .any(|rule| *rule == Rule(direction.clone(), color, other_color as u8))
            } else {
                false
            }
        )
}

