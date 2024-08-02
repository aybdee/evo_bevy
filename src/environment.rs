use bevy::{color::palettes::css::BLACK, prelude::*};
use bevy_prototype_lyon::prelude::*;
use rand::Rng;

use crate::utils::{generate_random_vec2, Grid2d};

#[derive(Component)]
pub struct Organism {
    color: Color,
    direction: Direction,
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

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
                Fill::color(Color::srgb(1.0, 100.0, 0.0)),
            ));

            self.organisms.set(grid_x, grid_y, organism.id().index());
        }
    }
}

fn calcululate_grid_position(
    width: f32,
    height: f32,
    organism_size: f32,
    x: f32,
    y: f32,
) -> (i32, i32) {
    let grid_x = ((x + (width / 2.0)) / organism_size).floor() as i32;
    let grid_y = ((y + (height / 2.0)) / organism_size).floor() as i32;
    (grid_x, grid_y)
}

pub fn environment_step(
    mut env: ResMut<Environment>,
    time: Res<Time>,
    mut simulation_speed: ResMut<SimulationSpeed>,
    mut organism_query: Query<(&mut Organism, &mut Transform)>,
) {
    if simulation_speed.timer.tick(time.delta()).just_finished() {
        for (mut organism, mut transform) in organism_query.iter_mut() {
            let current_position = transform.translation;
            let next_action = poll_organism_next_action(&mut env, &organism, current_position);

            // Calculate current grid position
            let (current_grid_x, current_grid_y) = calcululate_grid_position(
                env.width,
                env.height,
                env.organism_size,
                current_position.x,
                current_position.y,
            );

            match next_action {
                Action::Move(direction) => {
                    let (target_grid_x, target_grid_y) = match direction {
                        Direction::Left => (current_grid_x - 1, current_grid_y),
                        Direction::Right => (current_grid_x + 1, current_grid_y),
                        Direction::Up => (current_grid_x, current_grid_y + 1),
                        Direction::Down => (current_grid_x, current_grid_y - 1),
                    };

                    // Calculate new world position
                    let new_x = (target_grid_x as f32 * env.organism_size) - (env.width / 2.0)
                        + (env.organism_size / 2.0);
                    let new_y = (target_grid_y as f32 * env.organism_size) - (env.height / 2.0)
                        + (env.organism_size / 2.0);

                    // Update position
                    transform.translation.x = new_x;
                    transform.translation.y = new_y;
                    organism.direction = direction;
                }
                Action::Nothing => {}
            }
        }
    }
}

pub fn poll_organism_next_action(
    environment: &mut Environment,
    organism: &Organism,
    position: Vec3,
) -> Action {
    let distance_offset = environment.organism_size;
    match organism.direction {
        Direction::Left => {
            if position.x - distance_offset > -(environment.width / 2.0) {
                Action::Move(Direction::Left)
            } else {
                Action::Nothing
            }
        }
        Direction::Right => {
            if position.x + distance_offset < environment.width / 2.0 {
                Action::Move(Direction::Right)
            } else {
                Action::Nothing
            }
        }
        Direction::Up => {
            if position.y + distance_offset < environment.height / 2.0 {
                Action::Move(Direction::Up)
            } else {
                Action::Nothing
            }
        }
        Direction::Down => {
            if position.y - distance_offset > -(environment.height / 2.0) {
                Action::Move(Direction::Up)
            } else {
                Action::Nothing
            }
        }
    }
}
