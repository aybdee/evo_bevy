mod bezier;
mod environment;
mod gene;
mod graph;
mod neural;
mod utils;

use bevy::{
    color::palettes::{
        css::{BLACK, GHOST_WHITE, WHITE},
        tailwind::{GRAY_100, GRAY_200},
    },
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use environment::{environment_step, Environment, SimulationSpeed};
use graph::{DiagramConfig, Graph, GraphDiagram};
use neural::NeuralNet;

const ORGANISM_SIZE: f32 = 3.5;

fn setup(mut env: ResMut<Environment>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    //draw boundary
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Rectangle {
                extents: Vec2::new(env.width + env.thickness, env.height + env.thickness),
                ..shapes::Rectangle::default()
            }),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(env.x, env.y, 0.0)),
                ..default()
            },
            ..default()
        },
        Stroke::new(env.color, env.thickness),
    ));

    env.spawn_n_random_organisms(&mut commands, 1000);
}

fn main() {
    let environment = Environment::new(
        400.0,
        400.0,
        0.0,
        0.0,
        Color::Srgba(BLACK),
        2.0,
        0,
        ORGANISM_SIZE,
    );
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("evo"),
                    ..default()
                }),
                ..default()
            }),
            ShapePlugin,
        ))
        .insert_resource(ClearColor(Color::Srgba(GRAY_100)))
        .insert_resource(environment)
        .insert_resource(SimulationSpeed::new(0.0001))
        .add_systems(Startup, setup)
        .add_systems(Update, environment_step)
        .run();
}
