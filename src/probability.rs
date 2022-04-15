use itertools::Itertools;

use crate::*;
use std::{
     cmp::Ordering,
};

const R: i16 = 1;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    Conjunction(Box<Rule>, Box<Rule>),
    Neighbor(i16, i16, u8),
    NeighborhoodCount(u8, usize) // Neighborhood Counts did not help the score so they were removed.
}

fn generate_rules(pair: &Pair, x: usize, y: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity((R as usize * 2 + 1) ^ 4);
    
    let width = pair.input.len();
    let height = pair.input[0].len();

    let mut neighborhood_counts = [0; 10];

    // Generates neighbor rules.
    for xx in x as i16 - R..=x as i16 + R {
        for yy in y as i16 - R..=y as i16 + R {
            if xx > 0 && xx < width as i16 - 1 && yy > 0 && yy < height as i16 - 1 {
                let other_color = pair.input[xx as usize][yy as usize];
                let rule = Rule::Neighbor(xx - x as i16, yy - y as i16, other_color);
                rules.push(rule);

                // Updates neighborhood counts.
                neighborhood_counts[other_color as usize] += 1;
            }
        }
    }  

    // Generates neighborhood count rules.
    //for i in 0..10 {
        //rules.push(Rule::NeighborhoodCount(i, neighborhood_counts[i as usize]));
    //}

    // Generates conjunction rules.
    for i in 0..rules.len() {
        for j in i..rules.len() {
            let (left_rule, right_rule) = if rules[i] > rules[j] {
                (rules[i].clone(), rules[j].clone())
            } else {
                (rules[j].clone(), rules[i].clone())
            };

            rules.push(Rule::Conjunction(Box::new(left_rule), Box::new(right_rule)))
        }
    }
    

    rules
}


#[derive(Debug)]
pub struct Model {
    occurrence: HashMap<Rule, usize>,
    cooccurrence: HashMap<Rule, [usize; 10]>
}

pub fn induce(examples: &Vec<Pair>) -> Model {
    let mut model = Model {
        occurrence: HashMap::new(),
        cooccurrence: HashMap::new()
    };

    let mut rule_count = 0;

    for example in examples {
        let width = example.input.len();
        let height = example.input[0].len();

        for x in 0..width {
            for y in 0..height {
                let color = example.output[x][y] as usize;
                let rules = generate_rules(example, x, y);
                rule_count += rules.len();

                for rule in rules {
                    // Computes occurrence.
                    if let Some(occurrences) = model.occurrence.get_mut(&rule) {
                        *occurrences += 1;
                    } else {
                        model.occurrence.insert(rule.clone(), 1);
                    }

                    // Computes cooccurrence.
                    if let Some(cooccurrences) = model.cooccurrence.get_mut(&rule) {
                        cooccurrences[color] += 1;
                    } else {
                        let mut cooccurrences = [0; 10];
                        cooccurrences[color] = 1;
                        model.cooccurrence.insert(rule.clone(), cooccurrences);
                    }
                }
            }
        }
    }
    
    // NOTE: This output is for the .csv.
    print!("{rule_count},");

    model
}

const PROBABILITY_THRESHOLD: f32 = 0.0;

pub fn deduce(mut test: Pair, model: &Model) -> Pair {
    let width = test.input.len();
    let height = test.input[0].len();
    
    for x in 0..width {
        for y in 0..height {
            let mut probabilities = [0.0; 10];

            // Updates probabilities.
            for rule in generate_rules(&test, x, y) {
                let occurrences = *model.occurrence.get(&rule).unwrap_or(&0) as f32;
                let cooccurrences = model.cooccurrence.get(&rule).unwrap_or(&[0; 10]);
        
                for color in 0..10 {
                    let probability = cooccurrences[color] as f32 / occurrences;
                    
                    if probability.is_finite() {
                        probabilities[color] += probability;
                    }
                }
            }
            
            // Finds most likely output.
            let (output_color, probability) = probabilities
                .into_iter()
                .enumerate()
                .sorted_by(|(_, a), (_, b)| if b > a {Ordering::Greater} else {Ordering::Less} )
                .next()
                .unwrap();

            if probability > PROBABILITY_THRESHOLD {
                test.output[x][y] = output_color as u8;
            }
        }
    }

    test
}
