mod environment;
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
use neural::NeuralGraph;
use sickle_ui::{prelude::*, ui_commands::SetTextExt, SickleUiPlugin};

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

    env.spawn_n_random_organisms(&mut commands, 1000);

    let mut test_net = NeuralGraph::new(vec![2, 3, 2]);
    test_net.add_connection((0, 0), (2, 0), 1.0);
    test_net.add_connection((0, 1), (2, 0), 1.0);
    test_net.add_connection((0, 1), (1, 1), 1.0);
    let mut test_graph = Graph::from(test_net);
    test_graph.sort_edges();
    let diagram = test_graph.get_diagram(DiagramConfig {
        padding: 1.0,
        width: 100.0,
        height: 100.0,
        node_radius: 8.0,
        position: (0.0, 0.0),
        same_rank_scale: 0.8,
        arrow_thickness: 2.0,
    });
    diagram.spawn(&mut commands);
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
            SickleUiPlugin,
        ))
        .insert_resource(ClearColor(Color::Srgba(GRAY_100)))
        .insert_resource(environment)
        .insert_resource(SimulationSpeed::new(0.0001))
        .add_systems(Startup, setup)
        .add_systems(Update, environment_step)
        .run();
}
