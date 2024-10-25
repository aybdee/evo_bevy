use crate::{
    environment::Environment,
    environment::SimulationSpeed,
    organism::{Action, Direction, Organism, OrganismUpdateStore},
    utils::{vec2_to_i32, Grid2d},
};
use bevy::prelude::*;
use std::collections::HashMap;

pub fn poll_organism(
    organism_store: &HashMap<u32, Organism>,
    update_store: &mut OrganismUpdateStore,
    organism: Organism,
    environment: &Environment,
) -> Action {
    let organism_id = environment
        .organisms
        .get(organism.position.x as usize, organism.position.y as usize)
        .unwrap();

    let normalized_x = organism.position.x / environment.organisms.width as f32;
    let normalized_y = organism.position.y / environment.organisms.height as f32;

    if let Some(polled_direction) = organism.poll(vec![normalized_x, normalized_y]) {
        let intended_position =
            vec2_to_i32(calculate_new_position(polled_direction, organism.position));

        let in_bounds = match polled_direction {
            Direction::West => organism.position.x as i32 > 0,
            Direction::East => {
                (organism.position.x as i32) < (environment.organisms.width as i32) - 1
            }
            Direction::North => {
                (organism.position.y as i32) < (environment.organisms.height as i32) - 1
            }
            Direction::South => (organism.position.y as i32) > 0,
            Direction::SouthEast => {
                (organism.position.x as i32) < (environment.organisms.width as i32) - 1
                    && (organism.position.y as i32) > 0
            }

            Direction::SouthWest => {
                (organism.position.x as i32) > 0 && (organism.position.y as i32) > 0
            }

            Direction::NorthEast => {
                (organism.position.x as i32) < (environment.organisms.width as i32) - 1
                    && (organism.position.y as i32) < (environment.organisms.height as i32) - 1
            }

            Direction::NorthWest => {
                (organism.position.x as i32) > 0
                    && (organism.position.y as i32) < (environment.organisms.height as i32) - 1
            }
        };
        if !in_bounds {
            update_store.add_organism(
                *organism_id,
                vec2_to_i32(organism.position),
                Action::Nothing,
            );
            return Action::Nothing;
        }

        match update_store.get(intended_position) {
            Some((id, action)) => {
                if id == organism_id {
                    // Then update the store
                    *action
                } else {
                    //if organism already intends to move to that position do nothing
                    update_store.add_organism(
                        *organism_id,
                        vec2_to_i32(organism.position),
                        Action::Nothing,
                    );
                    Action::Nothing
                }
            }
            None => {
                //check if organism is currently at that position
                match environment
                    .organisms
                    .get(intended_position.0 as usize, intended_position.1 as usize)
                {
                    Some(id) => {
                        //poll to see if they're going to move
                        let action = poll_organism(
                            organism_store,
                            update_store,
                            organism_store.get(id).unwrap().clone(),
                            environment,
                        );

                        match action {
                            Action::Nothing => {
                                //occupying organism will remoain in that position
                                update_store.add_organism(
                                    *organism_id,
                                    vec2_to_i32(organism.position),
                                    Action::Nothing,
                                );
                                Action::Nothing
                            }

                            Action::Move(_) => {
                                //occupying organism will move away
                                update_store.add_organism(
                                    *organism_id,
                                    intended_position,
                                    Action::Move(polled_direction),
                                );
                                Action::Move(polled_direction)
                            }
                        }
                    }

                    None => {
                        update_store.add_organism(
                            *organism_id,
                            vec2_to_i32(calculate_new_position(
                                polled_direction,
                                organism.position,
                            )),
                            Action::Move(polled_direction),
                        );
                        Action::Move(polled_direction)
                    }
                }
            }
        }
    } else {
        update_store.add_organism(
            *organism_id,
            vec2_to_i32(organism.position),
            Action::Nothing,
        );
        Action::Nothing
    }

    //check if organism is in bounds
}

pub fn poll_organisms(
    environment: &mut Environment,
    organisms: Vec<(Organism, Entity)>,
) -> Vec<Action> {
    let mut directions = Vec::new();

    let organism_store: HashMap<u32, Organism> = organisms
        .iter()
        .map(|(organism, entity)| (entity.index(), organism.clone()))
        .collect();

    let mut update_store: OrganismUpdateStore = OrganismUpdateStore::new();

    //check if all ids  in environment.organisms are in organism_store

    for (organism, _) in organisms {
        let action = poll_organism(&organism_store, &mut update_store, organism, environment);
        directions.push(action);
    }

    directions
}

pub fn environment_step(
    mut env: ResMut<Environment>,
    time: Res<Time>,
    mut simulation_speed: ResMut<SimulationSpeed>,
    mut organism_query: Query<(&mut Organism, Entity, &mut Transform)>,
) {
    if simulation_speed.timer.tick(time.delta()).just_finished() {
        let organisms: Vec<(Organism, Entity)> = organism_query
            .iter()
            .map(|(organism, entity, _)| (organism.clone(), entity))
            .collect();

        let next_actions = poll_organisms(&mut env, organisms);

        env.organisms = Grid2d::new(env.organisms.width, env.organisms.height);

        for ((mut organism, entity, mut transform), action) in
            organism_query.iter_mut().zip(next_actions.iter())
        {
            match action {
                Action::Move(direction) => {
                    let (target_grid_x, target_grid_y) = match direction {
                        Direction::West => (organism.position.x - 1.0, organism.position.y),
                        Direction::East => (organism.position.x + 1.0, organism.position.y),
                        Direction::North => (organism.position.x, organism.position.y + 1.0),
                        Direction::South => (organism.position.x, organism.position.y - 1.0),
                        Direction::SouthEast => {
                            (organism.position.x + 1.0, organism.position.y - 1.0)
                        }
                        Direction::SouthWest => {
                            (organism.position.x - 1.0, organism.position.y - 1.0)
                        }

                        Direction::NorthEast => {
                            (organism.position.x + 1.0, organism.position.y + 1.0)
                        }

                        Direction::NorthWest => {
                            (organism.position.x - 1.0, organism.position.y + 1.0)
                        }
                    };

                    // Calculate new world position
                    let new_x = (target_grid_x * env.organism_size) - (env.width / 2.0)
                        + (env.organism_size / 2.0);
                    let new_y = (target_grid_y * env.organism_size) - (env.height / 2.0)
                        + (env.organism_size / 2.0);

                    // Update position
                    organism.position = Vec2::new(target_grid_x, target_grid_y);
                    transform.translation.x = new_x;
                    transform.translation.y = new_y;

                    // Update environment grid with new position
                    env.organisms.set(
                        target_grid_x as usize,
                        target_grid_y as usize,
                        entity.index(),
                    );

                    //update environment
                }
                Action::Nothing => {
                    env.organisms.set(
                        organism.position.x as usize,
                        organism.position.y as usize,
                        entity.index(),
                    );
                }
            }
        }
    }
}

pub fn calculate_new_position(direction: Direction, position: Vec2) -> Vec2 {
    match direction {
        Direction::West => Vec2::new(position.x - 1.0, position.y),
        Direction::East => Vec2::new(position.x + 1.0, position.y),
        Direction::North => Vec2::new(position.x, position.y + 1.0),
        Direction::South => Vec2::new(position.x, position.y - 1.0),
        Direction::SouthEast => Vec2::new(position.x + 1.0, position.y - 1.0),
        Direction::SouthWest => Vec2::new(position.x - 1.0, position.y - 1.0),
        Direction::NorthEast => Vec2::new(position.x + 1.0, position.y + 1.0),
        Direction::NorthWest => Vec2::new(position.x - 1.0, position.y + 1.0),
    }
}
