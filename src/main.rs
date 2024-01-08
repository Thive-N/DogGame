use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    asset,
    prelude::*,
};
use bevy::render::view::window;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: f32 = 200.0; // This is the player sprite size.
pub const NUMBER_OF_ENEMIES: usize = 1;

static mut SCORE:u64 = 0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, (setup, spawn_camera, spawn_player, spawn_enemy))
        .add_systems(Update, (player_movement, confine_player_movement, player_hit_enemy, text_color_system, fps_update_system, score_update_system))
        .run();
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Enemy {}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ScoreText;

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("dog.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(200., 200.)),
                ..default()
            },
            ..default()
        },
        Player {},
    ));
}

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

pub fn spawn_enemy(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let window = window_query.get_single().unwrap();
    
    let random_x = random::<f32>() * window.width();
    let random_y = random::<f32>() * window.height();
    
    commands.spawn((
        SpriteBundle {
        transform: Transform::from_xyz(random_x, random_y, 0.0),
        texture: asset_server.load("ball.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(100., 100.)),
            ..default()
        },
        ..default()
    },
    Enemy {},
));
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();

        let half_player_size = PLAYER_SIZE / 2.0; // 32.0
        let x_min = 0.0 + half_player_size;
        let x_max = window.width() - half_player_size;
        let y_min = 0.0 + half_player_size;
        let y_max = window.height() - half_player_size;

        let mut translation = player_transform.translation;

        // Bound the player x position
        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }
        // Bound the players y position.
        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        player_transform.translation = translation;
    }
}

pub fn player_hit_enemy(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok((enemy_entity, enemy_transform)) = enemy_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single() {
            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            if distance < PLAYER_SIZE / 2.0 {
                commands.entity(enemy_entity).despawn();
                spawn_enemy(commands, window_query, asset_server, enemy_query);
                unsafe {
                    SCORE += 1;
                }
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 60.0,
                    color: Color::GOLD,
            }),
        ]),
        FpsText,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 60.0,
                    color: Color::GOLD,
            }),
        ]).with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        ScoreText,
    ));
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn fps_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut fps_query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn score_update_system(
    mut score_query: Query<&mut Text, With<ScoreText>>,
) {
    for mut text in &mut score_query {
        unsafe {
            text.sections[1].value = format!("{value:.2}", value = SCORE);
        }
    }
}