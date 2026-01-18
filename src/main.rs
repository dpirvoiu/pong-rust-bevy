use bevy::{input::keyboard::Key, math::VectorSpace, prelude::*, window::WindowResolution};
use bevy_rapier2d::prelude::*;
use rand::Rng;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

const BALL_RADIUS: f32 = 25.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        ..RapierConfiguration::new(1.0)
    });

    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_event::<GameEvents>();

    app.add_systems(
        Startup,
        (spawn_camera, spawn_players, spawn_border, spawn_ball),
    );
    app.add_systems(Update, (move_paddle, detect_reset));
    app.add_systems(PostUpdate, reset_ball);
    app.run();
}

#[derive(Component)]
struct Paddle {
    move_up: KeyCode,
    move_down: KeyCode,
}

#[derive(Component, Clone, Copy)]
enum Player {
    Player1,
    Player2,
}

impl Player {
    fn start_speed(&self) -> Velocity {
        match self {
            Player::Player1 => Velocity::linear(Vec2::new(100.0, 0.0)),
            Player::Player2 => Velocity::linear(Vec2::new(-100.0, 0.0)),
        }
    }
}

fn spawn_border(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 2.0, 0.0)),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(WINDOW_WIDTH / 2.0, 3.0),
    ));

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 2.0, 0.0)),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(WINDOW_WIDTH / 2.0, 3.0),
    ));

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(WINDOW_WIDTH / 2.0, 0.0, 0.0)),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(3.0, WINDOW_HEIGHT / 2.0),
        Player::Player1,
        Sensor,
    ));

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-WINDOW_WIDTH / 2.0, 0.0, 0.0)),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(3.0, WINDOW_HEIGHT / 2.0),
        Player::Player2,
        Sensor,
    ));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_players(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                (-WINDOW_WIDTH / 2.0) + 20.0,
                0.0,
                0.0,
            )),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(10.0, 150.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Paddle {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
        },
        RigidBody::KinematicPositionBased,
        Collider::cuboid(5.0, 75.0),
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                (WINDOW_WIDTH / 2.0) - 20.0,
                0.0,
                0.0,
            )),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(10.0, 150.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Paddle {
            move_up: KeyCode::ArrowUp,
            move_down: KeyCode::ArrowDown,
        },
        RigidBody::KinematicPositionBased,
        Collider::cuboid(5.0, 75.0),
    ));
}

fn move_paddle(
    mut paddles: Query<(&mut Transform, &Paddle)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut pos, settings) in &mut paddles {
        if input.pressed(settings.move_up) {
            pos.translation.y += 100.0 * time.delta_seconds();
            pos.translation.y = pos
                .translation
                .y
                .clamp((-WINDOW_HEIGHT / 2.0) + 75.0, (WINDOW_HEIGHT / 2.0) - 75.0);
        }
        if input.pressed(settings.move_down) {
            pos.translation.y -= 100.0 * time.delta_seconds();
            pos.translation.y = pos
                .translation
                .y
                .clamp((-WINDOW_HEIGHT / 2.0) + 75.0, (WINDOW_HEIGHT / 2.0) - 75.0);
        }
    }
}

#[derive(Component)]
struct Ball;

fn spawn_ball(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bevy.png"),
            transform: Transform::from_translation(Vec3::new(-300.0, 0.0, 1.0)),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Ball,
        RigidBody::Dynamic,
        Collider::ball(BALL_RADIUS),
        ActiveEvents::COLLISION_EVENTS,
        CollidingEntities::default(),
        Velocity::linear(Vec2::new(100.0, 0.0)),
        Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
    ));
}

fn detect_reset(
    input: Res<ButtonInput<KeyCode>>,
    balls: Query<&CollidingEntities, With<Ball>>,
    goles: Query<&Player, With<Sensor>>,
    mut game_events: EventWriter<GameEvents>,
) {
    if input.just_pressed(KeyCode::Space) {
        let player = Player::Player1;

        game_events.send(GameEvents::ResetBall(player));
        return;
    }

    for ball in &balls {
        for hit in ball.iter() {
            if let Ok(player) = goles.get(hit) {
                game_events.send(GameEvents::ResetBall(*player));
            }
        }
    }
}

#[derive(Event)]
enum GameEvents {
    ResetBall(Player),
}

fn reset_ball(
    mut balls: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    mut game_events: EventReader<GameEvents>,
) {
    for events in game_events.read() {
        match events {
            GameEvents::ResetBall(player) => {
                for (mut ball, mut speed) in &mut balls {
                    ball.translation = Vec3::ZERO;
                    *speed = player.start_speed();
                }
            }
            _ => {}
        }
    }
}
