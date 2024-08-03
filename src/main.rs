mod environment;
mod utils;

use bevy::{color::palettes::css::BLACK, prelude::*};
use bevy_prototype_lyon::prelude::*;
use environment::{environment_step, Environment, SimulationSpeed};

const ORGANISM_SIZE: f32 = 2.5;

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

    env.spawn_n_random_organisms(commands, 100)
}

fn main() {
    let environment = Environment::new(
        400.0,
        400.0,
        0.0,
        0.0,
        Color::Srgba(BLACK),
        5.0,
        0,
        ORGANISM_SIZE,
    );
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin))
        .insert_resource(environment)
        .insert_resource(SimulationSpeed::new(0.0001))
        .add_systems(Startup, setup)
        .add_systems(Update, environment_step)
        .run();
}
