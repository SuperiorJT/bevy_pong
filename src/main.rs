use bevy::{
    ecs::Mut,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Bevy Pong".to_string(),
            ..Default::default()
        })
        .add_default_plugins()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(BallResetTimer(Timer::from_seconds(2.0, false)))
        .add_startup_system(setup.system())
        .add_system(player_input_system.system())
        .add_system(paddle_movement_system.system())
        .add_system(ball_movement_system.system())
        .add_system(ball_collision_system.system())
        .add_system(ball_reset_system.system())
        .run();
}

struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
}

enum Collider {
    Solid,
    Scoreable,
}

enum PaddleMovementState {
    Up,
    Down,
    None,
}

struct Paddle {
    move_state: PaddleMovementState,
    speed: f32,
}
struct Ball {
    velocity: Vec3,
}

struct BallResetTimer(Timer); // bool is used to prevent logic repeating when in finished state

struct Player {
    binds: KeyBinds,
}

struct Scoreboard(usize, usize);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // player 1
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(-600.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 120.0),
            },
            ..Default::default()
        })
        .with(Player {
            binds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
            },
        })
        .with(Paddle {
            move_state: PaddleMovementState::None,
            speed: 200.0,
        })
        .with(Collider::Solid)
        // player 2
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(600.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 120.0),
            },
            ..Default::default()
        })
        .with(Player {
            binds: KeyBinds {
                up: KeyCode::Up,
                down: KeyCode::Down,
            },
        })
        .with(Paddle {
            move_state: PaddleMovementState::None,
            speed: 200.0,
        })
        .with(Collider::Solid)
        // ball
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 20.0),
            },
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        });

    // Walls and goal areas
    commands
        // top
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(0.0, 345.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(1280.0, 30.0),
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom
        .spawn(SpriteComponents {
            material: materials.add(Color::WHITE.into()),
            translation: Translation(Vec3::new(0.0, -345.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(1280.0, 30.0),
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // left
        .spawn(SpriteComponents {
            translation: Translation(Vec3::new(-690.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(100.0, 720.0),
            },
            ..Default::default()
        })
        .with(Collider::Scoreable)
        // right
        .spawn(SpriteComponents {
            translation: Translation(Vec3::new(690.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(100.0, 720.0),
            },
            ..Default::default()
        })
        .with(Collider::Scoreable);

    // Divider
    commands.spawn(SpriteComponents {
        material: materials.add(Color::WHITE.into()),
        sprite: Sprite {
            size: Vec2::new(1.0, 720.0),
        },
        ..Default::default()
    });

    // Scoreboard
    let font_handle = asset_server.load("assets/fonts/bit5x3.ttf").unwrap();
    commands
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextComponents {
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        margin: Rect {
                            top: Val::Px(40.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "0 0".to_string(),
                        font: font_handle,
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    },
                    ..Default::default()
                })
                .with(Scoreboard(0, 0));
        });
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Paddle)>,
) {
    for (player, mut paddle) in &mut query.iter() {
        if keyboard_input.pressed(player.binds.up) {
            paddle.move_state = PaddleMovementState::Up
        } else if keyboard_input.pressed(player.binds.down) {
            paddle.move_state = PaddleMovementState::Down
        } else {
            paddle.move_state = PaddleMovementState::None
        }
    }
}

fn paddle_movement_system(time: Res<Time>, mut query: Query<(&Paddle, &mut Translation)>) {
    for (paddle, mut translation) in &mut query.iter() {
        match paddle.move_state {
            PaddleMovementState::Up => *translation.0.y_mut() += time.delta_seconds * paddle.speed,
            PaddleMovementState::Down => {
                *translation.0.y_mut() += time.delta_seconds * -1.0 * paddle.speed
            }
            _ => {}
        }

        *translation.0.y_mut() = f32::max(-270.0, f32::min(270.0, translation.0.y()))
    }
}

fn ball_movement_system(time: Res<Time>, mut query: Query<(&Ball, &mut Translation)>) {
    let delta_seconds = f32::min(0.2, time.delta_seconds);

    for (ball, mut translation) in &mut query.iter() {
        translation.0 += ball.velocity * delta_seconds;
    }
}

fn ball_reset_system(time: Res<Time>, mut timer: ResMut<BallResetTimer>, mut ball: Mut<Ball>) {
    timer.0.tick(time.delta_seconds);

    if timer.0.finished && timer.0.repeating {
        ball.velocity = 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize();
        timer.0.repeating = false;
    }
}

fn ball_collision_system(
    mut ball_reset_timer: ResMut<BallResetTimer>,
    mut ball_query: Query<(&mut Ball, &mut Sprite, &mut Translation)>,
    mut other_query: Query<(&Collider, &Translation, &Sprite)>,
    mut scoreboard_query: Query<(&mut Scoreboard, &mut Text)>,
) {
    for (mut ball, ball_sprite, mut ball_translation) in &mut ball_query.iter() {
        let ball_size = ball_sprite.size;
        let velocity = &mut ball.velocity;

        for (collider, other_translation, other_sprite) in &mut other_query.iter() {
            let collision = collide(
                ball_translation.0,
                ball_size,
                other_translation.0,
                other_sprite.size,
            );

            if let Some(collision) = collision {
                // Update scoreboard if we hit a goal
                if let &Collider::Scoreable = collider {
                    for (mut scoreboard, mut text) in &mut scoreboard_query.iter() {
                        if ball_translation.0.x() > 0.0 {
                            scoreboard.0 += 1;
                        }
                        if ball_translation.0.x() < 0.0 {
                            scoreboard.1 += 1;
                        }
                        text.value = format!("{} {}", scoreboard.0, scoreboard.1);
                        *velocity = Vec3::new(0.0, 0.0, 0.0);
                        *ball_translation = Translation(Vec3::new(0.0, 0.0, 0.0));
                        ball_reset_timer.0.repeating = true;
                        ball_reset_timer.0.reset();
                    }
                }

                match collision {
                    Collision::Left => {
                        if velocity.x() > 0.0 {
                            *velocity.x_mut() = -velocity.x();
                            *ball_translation.0.x_mut() = other_translation.0.x()
                                - other_sprite.size.x() / 2.0
                                - ball_size.x() / 2.0;
                        }
                    }
                    Collision::Right => {
                        if velocity.x() < 0.0 {
                            *velocity.x_mut() = -velocity.x();
                            *ball_translation.0.x_mut() = other_translation.0.x()
                                + other_sprite.size.x() / 2.0
                                + ball_size.x() / 2.0;
                        }
                    }
                    Collision::Top => {
                        if velocity.y() < 0.0 {
                            *velocity.y_mut() = -velocity.y();
                            *ball_translation.0.y_mut() = other_translation.0.y()
                                + other_sprite.size.y() / 2.0
                                + ball_size.y() / 2.0;
                        }
                    }
                    Collision::Bottom => {
                        if velocity.y() > 0.0 {
                            *velocity.y_mut() = -velocity.y();
                            *ball_translation.0.y_mut() = other_translation.0.y()
                                - other_sprite.size.y() / 2.0
                                - ball_size.y() / 2.0;
                        }
                    }
                }

                break;
            }
        }
    }
}
