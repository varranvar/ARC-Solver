use crate::*;
use std::{
    collections::{HashMap, HashSet, VecDeque},
};

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North = 0,
    West = 1,
    South = 2,
    East = 3,
    Up = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule(Direction, u8, u8);

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
                            Direction::South,
                            example.output[x][y],
                            example.output[x][y - 1],
                        ));
                    }

                    if y < height - 1 {
                        rules.insert(Rule(
                            Direction::North,
                            example.output[x][y],
                            example.output[x][y + 1],
                        ));
                    }

                    rules.insert(Rule(
                        Direction::Up,
                        example.input[x][y],
                        example.output[x][y],
                    ));
                }
            }

            rules
        })
        // Filters out non-constant rules.
        .reduce(|left_rules, right_rules| left_rules.union(&right_rules).cloned().collect())
        .unwrap()
}

type Domain = [[bool; 5]; 10];

pub fn deduce(mut test: Pair, rules: &Rules) -> Pair {
    let width = test.input.len();
    let height = test.input[0].len();
    let mut domains = vec![vec![[[true; 5]; 10]; height]; width];

    // Applies constraints from the input frame.
    for x in 0..width {
        for y in 0..height {
            let mut input_domain  = [[false; 5]; 10];
            input_domain[test.input[x][y] as usize] = [true; 5];

            if let Some(changed) = constrain(rules, Direction::Up, &input_domain, &mut domains[x][y]) {
                if changed {
                    propogate_changes(rules, &mut domains, x, y);
                }
            } else {
                panic!("Constraint solving failed at initial domain propogation.");
            }
        }
    }

    println!("{domains:?}");


    //println!("{domains:?}");

    // Searches for a valid solution.
    fn solve(
        rules: &Rules,
        domains: Vec<Vec<Domain>>,
        collapsed: usize,
    ) -> Option<Vec<Vec<Domain>>> {
        // Orders tile by entropy.
        let mut entropy = domains
            .iter()
            .enumerate()
            .map(|(x, column)| {
                column.iter().enumerate().map(move |(y, domain)| {
                    (
                        x,
                        y,
                        domain
                            .iter()
                            .filter(|supports| {
                                supports[0]
                                    || supports[1]
                                    || supports[2]
                                    || supports[3]
                                    || supports[4]
                            })
                            .count(),
                    )
                })
            })
            .flatten()
            .filter(|(_, _, entropy)| *entropy > 0)
            .collect::<Vec<(usize, usize, usize)>>();

        entropy.sort_unstable_by_key(|(_, _, entropy)| *entropy);

        // For every undecided tile:
        for (x, y, _) in entropy {
            //println!("Trying to collapse {x} {y}.");
            // For every open color:
            for color in 1..10 {
                // Skips if the color is already eliminated.
                if domains[x][y][color].iter().all(|s| !s) {
                    continue;
                }

                // Collapses the domain and propogate changes.
                let mut new_domains = domains.clone();
                
                new_domains[x][y] = [[false; 5]; 10];
                new_domains[x][y][color] = [true; 5];
                
                if !propogate_changes(rules, &mut new_domains, x, y) {
                    continue;
                }

                // Return if all tiles are collapsed.
                if collapsed >= domains.len() * domains[0].len() {
                    return Some(new_domains);
                }

                // Solves the next tile.
                if let Some(solution) = solve(rules, new_domains, collapsed + 1) {
                    return Some(solution);
                }
            }
        }

        None
    }


    fn propogate_changes(
        rules: &Rules,
        domains: &mut Vec<Vec<Domain>>,
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
                        let xx = x as i32 + x_offset;
                        let yy = y as i32 + y_offset;

                        if xx > 0 && xx < width as i32 - 1 && yy > 0 && yy < height as i32 - 1 {
                            // TODO: There is a quick-fix clone in here.
                            if let Some(changed) = constrain(
                                &rules,
                                Direction::West,
                                &domains[x][y].clone(),
                                &mut domains[xx as usize][yy as usize],
                            ) {
                                if changed {
                                    queue.push_back((xx as usize, yy as usize));
                                }
                            } else {
                                return false;
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
    fn constrain(
        rules: &Rules,
        direction: Direction,
        constrained_domain: &Domain,
        constraining_domain: &mut Domain,
    ) -> Option<bool> {
        let mut changed = false;
        for (constraining_color, constraining_supports) in
            constraining_domain.iter_mut().enumerate()
        {
            if constraining_supports[direction.clone() as usize] {
                if !constrained_domain
                    .iter()
                    .enumerate()
                    .any(|(constrained_color, _)| {
                        let rule = Rule(
                            direction.clone(),
                            constrained_color as u8,
                            constraining_color as u8,
                        );
                        rules.contains(&rule)
                    })
                {
                    constraining_supports[direction.clone() as usize] = false;

                    if constraining_supports.iter().all(|support| !support) {
                        changed = true;
                    }
                }
            }
        }

        if changed {
            if constraining_domain
                .iter()
                .all(|supports| supports.iter().all(|support| !support))
            {
                return None;
            } else {
                return Some(true);
            }
        }

        Some(false)
    }

    let solution = solve(rules, domains, 0).unwrap_or(vec![vec![[[false; 5]; 10]; height]; width]);

    for x in 0..width {
        for y in 0..height {
            let mut deduced_color = 0;

            for (color, supports) in solution[x][y].iter().enumerate() {
                if supports.iter().any(|support| *support) {
                    deduced_color = color;
                    break;
                }
            }

            test.output[x][y] = deduced_color as u8;
        }
    }

    test
}

// recoloring

type ColorOrdering = [u8; 10];

pub fn color_ordering(pair: &Pair) -> ColorOrdering {
    let mut counts = [0usize; 10];

    for x in 0..pair.input.len() {
        for y in 0..pair.input[0].len() {
            counts[pair.input[x][y] as usize] += 1;
            counts[pair.output[x][y] as usize] += 1;
        }
    }

    let mut ordering = counts
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, usize)>>();

    ordering.sort_by(|(_, a), (_, b)| a.cmp(b));

    ordering
        .into_iter()
        .map(|(color, _)| color as u8)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

pub fn recolor(pair: &mut Pair, ordering: &ColorOrdering) {
    // compute own color ordering and build a map
    let mut map = HashMap::new();
    for (i, color) in color_ordering(&pair).into_iter().enumerate() {
        map.insert(color, i);
    }

    // recolor
    for x in 0..pair.input.len() {
        for y in 0..pair.input[0].len() {
            pair.input[x][y] = ordering[*map.get(&pair.input[x][y]).unwrap()];
            pair.output[x][y] = ordering[*map.get(&pair.output[x][y]).unwrap()];
        }
    }
}
