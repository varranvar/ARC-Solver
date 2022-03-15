use crate::*;
use std::{
    collections::{ HashMap},
    fmt,
};
use itertools::Itertools;

pub const N: usize = 1;


type Rules = HashMap<(u8, u8), usize>;

fn induce(examples: &Vec<Pair>) -> Rules {
    examples
        .iter()
        .map(|example|{
            let mut rules: Rules = HashMap::new();
            let width = example.input.len();
            let height = example.input[0].len();
    
            for x in 0..width {
                for y in 0..height {
                    for nx in 0..1 + 2 * N {
                        for ny in 0..1 + 2 * N {
                        
                        }
                    }

                }
            }

            rules
        })
        .reduce(|a, b| 
            a
                .into_iter()
                .filter_map(|(key, value)|
                    if b.contains_key(&key) {
                        Some((key, value + b.get(&key).unwrap()))
                    } else {
                        None
                    }
                )
                .collect()
        )
        .unwrap()
}



pub fn deduce(mut test: Pair, rules: &Rules) -> Pair {
    let width = test.input.len();
    let height = test.input[0].len();

    let mut domains = vec![vec![[0; 10]; height]; width];

    // First round of confidence setting.
    for x in 0..width - N {
        for y in 0..height - N {

        }
    }

    test
}

const MAX_ITERATION_COUNT: usize = 1000;


