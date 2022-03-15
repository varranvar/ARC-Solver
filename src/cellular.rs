use crate::*;
use std::{
    collections::{ HashSet},
    fmt,
};
use itertools::Itertools;

pub const N: usize = 1;
const ANY: u8 = 0;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    input: [[u8; N]; N],
    output: [[u8; N]; N],
}

type Rules = Vec<Rule>;

pub fn induce(examples: &Vec<Pair>) -> Rules {
    let mut rules: Rules = Vec::new();
    for example in examples {
        let mut new_rules = Vec::new();
        let width = example.input.len();
        let height = example.input[0].len();

        // Generates new rules.
        for x in 0..width - N {
            for y in 0..height - N {
                let mut input = [[ANY; N]; N];
                let mut output = [[ANY; N]; N];
                for nx in 0..N {
                    for ny in 0..N {
                        input[nx][ny] = example.input[x + nx][y + ny];
                        output[nx][ny] = example.output[x + nx][y + ny];
                    }
                }

                new_rules.push(
                    Rule {
                        input,
                        output
                    }
                );
            }
        }

        // Sets rules as new rules if this is the first batch.
        if rules.len() == 0 {
            rules = new_rules;
            continue;
        }

        // Unifies new rules with old rules.
        let mut unified_rules = Vec::new();
        for new_rule in new_rules {
            for old_rule in &rules {
                // Unifies new and old rule.
                let mut input = [[ANY; N]; N];
                let mut output = [[ANY; N]; N];

                for nx in 0..N {
                    for ny in 0..N {
                        let new_input = new_rule.input[nx][ny];
                        let old_input = old_rule.input[nx][ny];
                        let new_output = new_rule.output[nx][ny];
                        let old_output = old_rule.output[nx][ny];

                        if old_input == ANY || new_input != old_input {
                            input[nx][ny] = ANY;
                        } else {
                            input[nx][ny] = new_input;
                        }

                        if old_output == ANY || new_output != old_output {
                            output[nx][ny] = ANY;
                        } else {
                            output[nx][ny] = new_output;
                        }
                    }
                }
                unified_rules.push(Rule{
                    input,
                    output
                });
            }
        }

        
        rules = unified_rules.into_iter()
            .filter(|rule| 
                rule.input
                    .into_iter()
                    .map(|column| column.into_iter().filter(|color| *color != ANY).count())
                    .sum::<usize>()
                +
                rule.output
                    .into_iter()
                    .map(|column| column.into_iter().filter(|color| *color != ANY).count())
                    .sum::<usize>()
                >= N
            )
            .unique()
            .collect();
    }

    // TODO: Order rules based on the number of non-ANY cells.

    rules
}



pub fn deduce(mut test: Pair, rules: &Rules) -> Pair {
    let width = test.input.len();
    let height = test.input[0].len();

    for x in 0..width - N {
        for y in 0..height - N {
            for rule in rules {
                // Sets the output if the input matches with the rule.
                if (0..N)
                    .zip(0..N)
                    .all(|(nx, ny)| {
                        rule.input[nx][ny] == ANY || rule.input[nx][ny] == test.input[x + nx][y + ny]
                    }) 
                {
                    for nx in 0..N {
                        for ny in 0..N {
                            if rule.output[nx][ny] != ANY && test.output[x + nx][y + ny] == ANY {
                                test.output[x + nx][y + ny] = rule.output[nx][ny];
                            }
                        }
                    }
                }
            }
        }
    }

    test
}

const MAX_ITERATION_COUNT: usize = 1000;



// Utility

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();

        for y in 0..N {
            for x in 0..N {
                string.push_str(&cell_to_string(self.input[x][y])[..]);
            }

            if y == N / 2 {
                string.push_str(" -> ");
            } else {
                string.push_str("    ");
            }

            
            for x in 0..N {
                string.push_str(&cell_to_string(self.output[x][y])[..]);
            }
            
            string.push_str("\n");
        }
        
        write!(f, "{string}")
    }
}