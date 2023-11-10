use std::default;

use bevy::audio::PlaybackMode;
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomSettings};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowTheme};
use bevy::*;
use rand::Rng;
use anyhow::Result;
use bevy::ecs::query::QuerySingleError;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(GameOver(GameState::Continue))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pong".into(),
                        resolution: (1600., 800.).into(),
                        present_mode: PresentMode::AutoVsync,
                        // Tells wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        enabled_buttons: bevy::window::EnabledButtons {
                            maximize: false,
                            minimize: false,
                            ..Default::default()
                        },
                        // This will spawn an invisible window
                        // The window will be made visible in the make_visible() system after 3 frames.
                        // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                        visible: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, start_pong)
        .add_systems(
            Update,
            (do_player_bat_movement, do_bot_movement, do_ball_movement),
        )
        .run();
}

#[derive(Component)]
pub enum DirectionY {
    Up,
    Down,
}

#[derive(Component)]
pub struct Bounces(u32);

#[derive(PartialEq, Component)]
pub enum DirectionX {
    Left,
    Right,
}

// I dont even know.... it wasnt working so i gave both the bats names and now they work
#[derive(Component)]
pub struct Name(String);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct BallFact(f32);

#[derive(Component)]
pub struct Bat;

#[derive(Component)]
pub struct Acceleration(f32);

#[derive(Component)]
pub struct MaxVelocity(f32);

#[derive(Component)]
pub struct BounceSound1;

#[derive(Component)]
pub struct BounceSound2;

#[derive(Component)]
pub struct DeathSound;

#[derive(Component)]
pub struct Velocity(Vec2);

fn start_pong(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings {
            intensity: 0.077,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 1.0,
            high_pass_frequency: 1.0,
            prefilter_settings: Default::default(),
            composite_mode: BloomCompositeMode::EnergyConserving,
        }, // 3. Enable bloom for the camera
    ));
    commands.spawn(SpriteBundle {
        texture: asset_server.load("game_sprites/middle.png"),
        transform: Transform::from_scale(Vec3 {
            x: 3.0,
            y: 3.0,
            z: 1.0,
        }),
        ..default()
    });
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
        Name(String::from("Sebastien")),
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
        Name(String::from("Gemma")),
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("game_sprites/ball.png"),
            transform: Transform::from_scale(Vec3 {
                x: 3.0,
                y: 3.0,
                z: 1.0,
            }),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(0.0, 0.0)),
        DirectionY::Up,
        DirectionX::Left,
        BallFact(2.0),
        Bounces(0),
    ));
    commands.spawn((AudioBundle {
        source: asset_server.load("sounds/totally_not_stolen_portal_slash_half-life2_sound_effects/energy_sing_loop4.ogg"),
        settings : PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Default::default(),
            speed: 1.0,
            paused: false,
            spatial: false,
        }

    },
    Loop, 
)



);

    //commands.get_entity(looping).
}

pub fn get_ball_fac(bounces: &mut u32, bloom: &mut BloomSettings) -> f32 {
    let mut val = 2.0;

    // this *bounces > 14 is to make the condition true.... at 16 bounces. only made it work at 17 bounces... for some reason. I.. Idk why
    if *bounces > 14 {
        val = 12.0
    }
    if (7..15).contains(&bounces.to_owned()) {
        val = 8.0;
    }
    if (4..7).contains(&bounces.to_owned()) {
        val = 6.3;
    }
    if (1..4).contains(&bounces.to_owned()) {
        val = 4.5;
    }
    *bounces += 1;
    bloom.intensity = 0.3;
    return val;
}

#[derive(PartialEq)]
pub enum GameState {
    Over,
    Continue
}
#[derive(Resource)]
pub struct GameOver(GameState);

#[derive(Component)]
pub struct Loop;

fn do_ball_movement(
    _time: Res<Time>,
    mut camera: Query<&mut BloomSettings, With<Camera>>,
    mut ball: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut DirectionY,
            &mut DirectionX,
            &mut BallFact,
            &mut Bounces,
        ),
        (With<Ball>, Without<Bat>),
    >,
    mut player_bat: Query<&mut Transform, (With<Bat>, Without<Ball>, With<Player>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    music_controller : Query<&AudioSink, With<Loop>>,
    mut game_over : ResMut<GameOver>
) {
    if game_over.0 == GameState::Continue {
    let mut bloom = camera.get_single_mut().unwrap();
    let num = rand::thread_rng().gen_range(0..2);
    let mut bounces_b : u32;
    for (mut trans, mut velocity, mut diry, mut dirx, mut fact, mut bounces) in &mut ball {
        bounces_b = bounces.0;
        if bloom.intensity > 0.077 {
            bloom.intensity -= 0.004
        }

        if velocity.0.x + velocity.0.y == 0.0 {
            velocity.0.x = 20.0;
            velocity.0.y = 20.0;
        }

        //velocity.0.x += trans.translation.x;
        //velocity.0.y += trans.translation.y;

        velocity.0 = velocity.0.normalize();
        velocity.0.x *= fact.0;
        velocity.0.y *= fact.0;
        if trans.translation.y < -400.0 {
            *diry = DirectionY::Up;
            fact.0 = get_ball_fac(&mut bounces.0, &mut bloom);
        } else if trans.translation.y > 400.0 {
            *diry = DirectionY::Down;
            fact.0 = get_ball_fac(&mut bounces.0, &mut bloom);
        }
        if trans.translation.x > 700.0 {
            *dirx = DirectionX::Left;
            fact.0 = get_ball_fac(&mut bounces.0, &mut bloom);
        } else if trans.translation.x < -800.0 {
            if let Ok(sink) = music_controller.get_single() {
                sink.pause();
                commands.spawn(AudioBundle {
                    source: asset_server.load(
                        "sounds/totally_not_stolen_portal_slash_half-life2_sound_effects/energy_sing_explosion2.ogg",
                    ),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Once,
                        volume: Default::default(),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                    },
                });
                game_over.0 = GameState::Over
            }
        }
        for transb in &mut player_bat {
            if (-712.0..-699.0).contains(&trans.translation.x)
                && *dirx == DirectionX::Left
                && (transb.translation.y - 43.0..transb.translation.y + 42.0)
                    .contains(&trans.translation.y)
            {
                *dirx = DirectionX::Right;
                fact.0 = get_ball_fac(&mut bounces.0, &mut bloom);
            }
        }
        match *dirx {
            DirectionX::Left => {
                trans.translation.x -= velocity.0.x;
            }
            DirectionX::Right => {
                trans.translation.x += velocity.0.x;
            }
        }
        match *diry {
            DirectionY::Down => {
                trans.translation.y -= velocity.0.y;
            }
            DirectionY::Up => {
                trans.translation.y += velocity.0.y;
            }
        }

        // bad approach but icba anymore

        if bounces_b < bounces.0 {
            if num > 0 {
                commands.spawn(AudioBundle {
                    source: asset_server.load(
                        "sounds/totally_not_stolen_portal_slash_half-life2_sound_effects/energy_bounce1.ogg",
                    ),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Once,
                        volume: Default::default(),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                    },
                });
            }else {
                commands.spawn(AudioBundle {
                    source: asset_server.load(
                        "sounds/totally_not_stolen_portal_slash_half-life2_sound_effects/energy_bounce2.ogg",
                    ),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Once,
                        volume: Default::default(),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                    },
                });
            }
        }

        //println!("{}    {}", trans.translation.x, velocity.0.x)
    }
}
}

fn do_bot_movement(
    mut bot_bat: Query<(&mut Transform, &mut Name), (Without<Player>, Without<Ball>)>,
    mut ball: Query<&mut Transform, (With<Ball>, Without<Bat>)>,
) {
    for (mut transform, _name) in &mut bot_bat {
        for transformb in &mut ball {
            transform.translation = Vec2::new(700.0, transformb.translation.y).extend(0.0);
        }
    }
}

fn do_player_bat_movement(
    mut player_bat: Query<&mut Transform, With<Player>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    for mut transform in &mut player_bat {
        let (camera, cam_trans) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(cam_trans, cursor))
            .map(|ray| ray.origin.truncate())
        {
            transform.translation = Vec2::new(-700.0, world_position.y).extend(0.0);
        }
    }

    /*
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
    transform.translation.y = velocity.0;

    if transform.translation.y > 200. {
        *dir = Direction::Down
    } else if transform.translation.y < -200. {
        *dir = Direction::Up;

    }

     */
}
