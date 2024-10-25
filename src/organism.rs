//organisms functionality

use crate::{gene::Genome, neural::NeuralNet, utils::bernoulli_trial};
use bevy::prelude::*;

use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum Direction {
    West,
    East,
    North,
    South,
    SouthWest,
    SouthEast,
    NorthWest,
    NorthEast,
}

#[derive(Copy, Clone)]
pub enum Action {
    Move(Direction),
    Nothing,
}

#[derive(Component, Clone)]
pub struct Organism {
    pub brain: NeuralNet,
    pub genome: Genome,
    pub position: Vec2,
}

fn resolve_directions(directions: Vec<Direction>) -> Option<Direction> {
    // Takes a list of directions and returns the final direction
    if directions.len() == 1 {
        return Some(directions[0]);
    }

    let (mut dir_x, mut dir_y) = (0, 0);

    // Accumulate direction components
    for direction in directions.iter() {
        match direction {
            Direction::West => dir_x -= 1,
            Direction::East => dir_x += 1,
            Direction::North => dir_y += 1,
            Direction::South => dir_y -= 1,
            _ => {}
        }
    }

    match (dir_x, dir_y) {
        (0, 0) => None,
        (0, y) if y > 0 => Some(Direction::North),
        (0, _) => Some(Direction::South),
        (x, 0) if x > 0 => Some(Direction::East),
        (_, 0) => Some(Direction::West),
        (x, y) if x > 0 && y > 0 => Some(Direction::NorthEast),
        (x, y) if x > 0 && y < 0 => Some(Direction::SouthEast),
        (x, y) if x < 0 && y > 0 => Some(Direction::NorthWest),
        _ => Some(Direction::SouthWest),
    }
}

//input structure
//0 - Px - x position of the organism
//1 - Py - y position of the organism

//output structure
//0 - Mx - move in x direction
//1 - My - move in y direction

impl Organism {
    pub fn poll(&self, input: Vec<f32>) -> Option<Direction> {
        let output = self.brain.forward(input);
        let mx = output[0];
        let my = output[1];

        let mut move_directions: Vec<Direction> = vec![];

        if mx > 0.0 && bernoulli_trial(mx) {
            move_directions.push(Direction::East)
        } else if mx < 0.0 && bernoulli_trial(-mx) {
            move_directions.push(Direction::West)
        }

        if my > 0.0 && bernoulli_trial(my) {
            move_directions.push(Direction::North)
        } else if my < 0.0 && bernoulli_trial(-my) {
            move_directions.push(Direction::South)
        }

        resolve_directions(move_directions)
    }
}

pub struct OrganismUpdateStore {
    pub updates: HashMap<(i32, i32), (u32, Action)>,
}

impl OrganismUpdateStore {
    pub fn new() -> Self {
        OrganismUpdateStore {
            updates: HashMap::new(),
        }
    }

    pub fn add_organism(&mut self, id: u32, position: (i32, i32), action: Action) {
        self.updates.insert(position, (id, action));
    }

    pub fn get(&self, position: (i32, i32)) -> Option<&(u32, Action)> {
        self.updates.get(&position)
    }
}
