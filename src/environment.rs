use crate::utils::{generate_random_vec2, vec2_to_i32, Grid2d};
use bevy::{
    color::palettes::{css::RED, tailwind::RED_100},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use rand::Rng;
use std::collections::HashMap;
#[derive(Component, Copy, Clone)]
pub struct Organism {
    color: Color,
    direction: Direction,
    position: Vec2,
}

#[derive(Copy, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Copy, Clone)]
pub enum Action {
    Move(Direction),
    Nothing,
}

#[derive(Resource)]
pub struct SimulationSpeed {
    pub timer: Timer,
}
impl SimulationSpeed {
    pub fn new(speed: f32) -> Self {
        SimulationSpeed {
            timer: Timer::from_seconds(speed, TimerMode::Repeating),
        }
    }
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        SimulationSpeed {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct Environment {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub thickness: f32,
    pub num_organisms: usize,
    pub organism_size: f32,
    pub organisms: Grid2d<u32>,
}

impl Environment {
    pub fn new(
        width: f32,
        height: f32,
        x: f32,
        y: f32,
        color: Color,
        thickness: f32,
        num_organisms: usize,
        organism_size: f32,
    ) -> Self {
        let organisms = Grid2d::new(
            (width / organism_size).ceil() as usize,
            (height / organism_size).ceil() as usize,
        );
        Environment {
            width,
            height,
            x,
            y,
            color,
            thickness,
            num_organisms,
            organism_size,
            organisms,
        }
    }

    pub fn spawn_n_random_organisms(&mut self, mut commands: Commands, n: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..(n) {
            let rand_direction = Direction::Left;
            let grid_x = rng.gen_range(0..self.organisms.width);
            let grid_y = rng.gen_range(0..self.organisms.height);

            // Convert grid coordinates to world coordinates
            let world_x = (grid_x as f32 * self.organism_size) + self.x - (self.width / 2.0)
                + (self.organism_size / 2.0);
            let world_y = (grid_y as f32 * self.organism_size) + self.y - (self.height / 2.0)
                + (self.organism_size / 2.0);

            let organism = commands.spawn((
                Organism {
                    color: Color::srgb(1.0, 100.0, 0.0),
                    direction: rand_direction,
                    position: Vec2::new(grid_x as f32, grid_y as f32),
                },
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shapes::Rectangle {
                        extents: Vec2::splat(self.organism_size),
                        ..shapes::Rectangle::default()
                    }),
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(world_x, world_y, 1.0)),
                        ..default()
                    },
                    ..default()
                },
                Fill::color(Color::Srgba(RED)),
            ));

            self.organisms.set(grid_x, grid_y, organism.id().index());
        }
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

    pub fn add_organism(&mut self, id: u32, positon: (i32, i32), action: Action) {
        self.updates.insert(positon, (id, action));
    }

    pub fn get(&self, position: (i32, i32)) -> Option<&(u32, Action)> {
        self.updates.get(&position)
    }
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
            .map(|(organism, entity, _)| (*organism, entity))
            .collect();

        let next_actions = poll_organisms(&mut env, organisms);

        for ((mut organism, entity, mut transform), action) in
            organism_query.iter_mut().zip(next_actions.iter())
        {
            match action {
                Action::Move(direction) => {
                    let (target_grid_x, target_grid_y) = match direction {
                        Direction::Left => (organism.position.x - 1.0, organism.position.y),
                        Direction::Right => (organism.position.x + 1.0, organism.position.y),
                        Direction::Up => (organism.position.x, organism.position.y + 1.0),
                        Direction::Down => (organism.position.x, organism.position.y - 1.0),
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
                    organism.direction = *direction;

                    //update environment
                }
                Action::Nothing => {}
            }
        }
    }
}

pub fn calculate_new_position(direction: Direction, position: Vec2) -> Vec2 {
    match direction {
        Direction::Left => Vec2::new(position.x - 1.0, position.y),
        Direction::Right => Vec2::new(position.x + 1.0, position.y),
        Direction::Up => Vec2::new(position.x, position.y + 1.0),
        Direction::Down => Vec2::new(position.x, position.y - 1.0),
    }
}

// HashMap
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

    let intended_position = vec2_to_i32(calculate_new_position(
        organism.direction,
        organism.position,
    ));

    //check if organism is in bounds
    let in_bounds = match organism.direction {
        Direction::Left => organism.position.x as i32 > 0,
        Direction::Right => (organism.position.x as i32) < (environment.organisms.width as i32) - 1,
        Direction::Up => (organism.position.y as i32) < (environment.organisms.height as i32) - 1,
        Direction::Down => (organism.position.y as i32) > 0,
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
                //organism has already been polled
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
                        organism_store.get(&id).unwrap().clone(),
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
                                Action::Move(organism.direction),
                            );
                            Action::Move(organism.direction)
                        }
                    }
                }

                None => {
                    update_store.add_organism(
                        *organism_id,
                        vec2_to_i32(calculate_new_position(
                            organism.direction,
                            organism.position,
                        )),
                        Action::Move(organism.direction),
                    );
                    Action::Move(organism.direction)
                }
            }
        }
    }
}

pub fn poll_organisms(
    environment: &mut Environment,
    organisms: Vec<(Organism, Entity)>,
) -> Vec<Action> {
    let mut directions = Vec::new();

    let organism_store: HashMap<u32, Organism> = organisms
        .iter()
        .map(|(organism, entity)| (entity.index(), *organism))
        .collect();

    let mut update_store: OrganismUpdateStore = OrganismUpdateStore::new();
    //check if all ids  in environment.organisms are in organism_store

    for (organism, _) in organisms {
        let action = poll_organism(&organism_store, &mut update_store, organism, environment);
        directions.push(action);
    }

    //update environment
    environment.organisms = Grid2d::new(environment.organisms.width, environment.organisms.height);
    for ((x, y), (id, _)) in update_store.updates.iter() {
        environment.organisms.set(*x as usize, *y as usize, *id);
    }
    directions
}
