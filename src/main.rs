use bevy::prelude::*;
use num::clamp;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, start_pong)
        .add_systems(Update, do_player_bat_movement)
        .run();
}

#[derive(Component)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Bat;

#[derive(Component)]
pub struct Acceleration(f32);

#[derive(Component)]
pub struct MaxVelocity(f32);

#[derive(Component)]
pub struct Velocity(f32);

fn start_pong(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("game_sprites/bat.png"),
            transform: Transform::from_scale(Vec3 {
                x: 3.0,
                y: 3.0,
                z: 1.0,
            }),
            ..default()
        },
        Player,
        Bat,
        Acceleration(13.),
        MaxVelocity(2000.),
        Velocity(2.),
        Direction::Up,
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("game_sprites/bat.png"),
            transform: Transform::from_scale(Vec3 {
                x: 3.0,
                y: 3.0,
                z: 1.0,
            }),
            ..default()
        },
        Bat,
    ));
}

fn do_player_bat_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut MaxVelocity,
            &mut Acceleration,
            &mut Velocity,
            &mut Direction,
        ),
        With<Player>,
    >,
) {
    for (mut transform, max_velocity, acceleration, mut velocity, mut dir) in &mut query {
        match *dir {
            Direction::Up => {
                velocity.0 += clamp(
                    acceleration.0 * time.delta_seconds(),
                    f32::MIN,
                    max_velocity.0,
                )
            }
            Direction::Down => {
                velocity.0 += clamp(
                    -acceleration.0 * time.delta_seconds(),
                    f32::MIN,
                    max_velocity.0,
                )
            }
        }

        println!("{} ms^-2         {:?} s", velocity.0, time.delta_seconds());
        transform.translation.y = velocity.0; //* time.delta_seconds();

        if transform.translation.y > 200. {
            *dir = Direction::Down
        } else if transform.translation.y < -200. {
            *dir = Direction::Up;
        }
    }
}
