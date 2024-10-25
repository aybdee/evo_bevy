use crate::{
    gene::Genome,
    neural::{NeuralNet, WEIGHT_RANGE},
    organism::{Action, Direction, Organism},
    utils::{vec2_to_i32, Grid2d},
};

use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;
use rand::Rng;
use std::collections::HashMap;

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

    pub fn spawn_organism_n(&mut self, commands: &mut Commands, mut organism: Organism, n: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..n {
            let grid_x = rng.gen_range(0..self.organisms.width);
            let grid_y = rng.gen_range(0..self.organisms.height);
            organism.position = Vec2::new(grid_x as f32, grid_y as f32);
            self.spawn_organism(commands, organism.clone(), (grid_x, grid_y));
        }
    }

    pub fn spawn_organism(
        &mut self,
        commands: &mut Commands,
        organism: Organism,
        position: (usize, usize),
    ) {
        let organism_color = organism.genome.get_color();

        // Convert grid coordinates to world coordinates
        let world_x = (position.0 as f32 * self.organism_size) + self.x - (self.width / 2.0)
            + (self.organism_size / 2.0);
        let world_y = (position.1 as f32 * self.organism_size) + self.y - (self.height / 2.0)
            + (self.organism_size / 2.0);

        let organism_entity = commands.spawn((
            organism,
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
            Fill::color(Color::hsl(
                organism_color.hue as f32,
                organism_color.saturation,
                0.5,
            )),
        ));

        self.organisms
            .set(position.0, position.1, organism_entity.id().index());
    }

    pub fn spawn_n_random_organisms(&mut self, commands: &mut Commands, n: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..(n) {
            let grid_x = rng.gen_range(0..self.organisms.width);
            let grid_y = rng.gen_range(0..self.organisms.height);

            let mut organism_brain = NeuralNet::new(vec![2, 1, 2]);
            organism_brain.init_random_connections(4, (-WEIGHT_RANGE, WEIGHT_RANGE));
            let organism_genome: Genome = organism_brain.clone().into();
            let organism = Organism {
                brain: organism_brain,
                genome: organism_genome,
                position: Vec2::new(grid_x as f32, grid_y as f32),
            };

            self.spawn_organism(commands, organism, (grid_x, grid_y))
        }
    }
}
