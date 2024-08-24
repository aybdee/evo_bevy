mod environment;
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
use sickle_ui::{prelude::*, ui_commands::SetTextExt, SickleUiPlugin};

const ORGANISM_SIZE: f32 = 2.5;

fn draw_graph(mut commands: Commands) {}

fn setup(mut env: ResMut<Environment>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // commands.ui_builder(UiRoot).column(|column| {
    //     column
    //         .style()
    //         .position_type(PositionType::Absolute)
    //         .justify_content(JustifyContent::Start)
    //         .padding(UiRect::new(
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //         ))
    //         .width(Val::Percent(25.0))
    //         .right(Val::Px(0.0))
    //         .top(Val::Px(0.0))
    //         .background_color(Color::Srgba(GHOST_WHITE));
    //
    //     column
    //         .label(LabelConfig::default())
    //         .style()
    //         .align_self(AlignSelf::Start)
    //         .justify_self(JustifySelf::Start)
    //         .entity_commands()
    //         .set_text(
    //             "View panel",
    //             Some(TextStyle {
    //                 color: Color::Srgba(BLACK),
    //                 font_size: 30.0,
    //                 ..default()
    //             }),
    //         );
    // });
    //
    // commands.ui_builder(UiRoot).column(|column| {
    //     column
    //         .style()
    //         .position_type(PositionType::Absolute)
    //         .justify_content(JustifyContent::Start)
    //         .padding(UiRect::new(
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //             Val::Px(5.0),
    //         ))
    //         .width(Val::Percent(25.0))
    //         .left(Val::Px(0.0))
    //         .top(Val::Px(0.0))
    //         .background_color(Color::Srgba(GHOST_WHITE));
    //
    //     column
    //         .label(LabelConfig::default())
    //         .style()
    //         .align_self(AlignSelf::Start)
    //         .justify_self(JustifySelf::Start)
    //         .entity_commands()
    //         .set_text(
    //             "Evo",
    //             Some(TextStyle {
    //                 color: Color::Srgba(BLACK),
    //                 font_size: 30.0,
    //                 ..default()
    //             }),
    //         );
    // });

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
