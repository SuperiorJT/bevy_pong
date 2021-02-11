use bevy::{
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
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(BallResetTimer(Timer::from_seconds(2.0, false), false))
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

struct BallResetTimer(Timer, bool); // bool is used to prevent logic repeating when in finished state

struct Player {
    binds: KeyBinds,
}

struct Scoreboard(usize, usize);

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // player 1
        .spawn(SpriteBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::new(-600.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 120.0),
                ..Default::default()
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
        .spawn(SpriteBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::new(600.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 120.0),
                ..Default::default()
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
        .spawn(SpriteBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 20.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        });

    // Walls and goal areas
    commands
        // top
        .spawn(SpriteBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 345.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(1280.0, 30.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom
        .spawn(SpriteBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::new(0.0, -345.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(1280.0, 30.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // left
        .spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-690.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(100.0, 720.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Collider::Scoreable)
        // right
        .spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(690.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(100.0, 720.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Collider::Scoreable);

    // Divider
    commands.spawn(SpriteBundle {
        material: materials.add(Color::WHITE.into()),
        sprite: Sprite {
            size: Vec2::new(1.0, 720.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Scoreboard
    let font_handle = asset_server.load("fonts/bit5x3.ttf");
    commands
        .spawn(NodeBundle {
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
                .spawn(TextBundle {
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
                            ..Default::default()
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
    for (player, mut paddle) in query.iter_mut() {
        if keyboard_input.pressed(player.binds.up) {
            paddle.move_state = PaddleMovementState::Up
        } else if keyboard_input.pressed(player.binds.down) {
            paddle.move_state = PaddleMovementState::Down
        } else {
            paddle.move_state = PaddleMovementState::None
        }
    }
}

fn paddle_movement_system(time: Res<Time>, mut query: Query<(&Paddle, &mut Transform)>) {
    for (paddle, mut transform) in query.iter_mut() {
        match paddle.move_state {
            PaddleMovementState::Up => {
                transform.translation.y += time.delta_seconds() * paddle.speed
            }
            PaddleMovementState::Down => {
                transform.translation.y += time.delta_seconds() * -1.0 * paddle.speed
            }

            PaddleMovementState::None => {}
        }

        transform.translation.y = f32::max(-270.0, f32::min(270.0, transform.translation.y))
    }
}

fn ball_movement_system(time: Res<Time>, mut query: Query<(&Ball, &mut Transform)>) {
    let delta_seconds = f32::min(0.2, time.delta_seconds());

    for (ball, mut transform) in query.iter_mut() {
        transform.translation += ball.velocity * delta_seconds;
    }
}

fn ball_reset_system(
    time: Res<Time>,
    mut timer: ResMut<BallResetTimer>,
    mut balls: Query<&mut Ball>,
) {
    timer.0.tick(time.delta_seconds());

    if timer.0.finished() && timer.1 {
        for mut ball in balls.iter_mut() {
            ball.velocity = 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize();
        }
        timer.1 = false;
    }
}

fn ball_collision_system(
    mut ball_reset_timer: ResMut<BallResetTimer>,
    mut ball_query: Query<(&mut Ball, &mut Sprite, &mut Transform)>,
    other_query: Query<(&Collider, &Transform, &Sprite)>,
    mut scoreboard_query: Query<(&mut Scoreboard, &mut Text)>,
) {
    for (mut ball, ball_sprite, mut ball_transform) in ball_query.iter_mut() {
        let ball_size = ball_sprite.size;
        let velocity = &mut ball.velocity;

        for (collider, other_transform, other_sprite) in other_query.iter() {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                other_transform.translation,
                other_sprite.size,
            );

            if let Some(collision) = collision {
                // Update scoreboard if we hit a goal
                if let &Collider::Scoreable = collider {
                    for (mut scoreboard, mut text) in scoreboard_query.iter_mut() {
                        if ball_transform.translation.x > 0.0 {
                            scoreboard.0 += 1;
                        }
                        if ball_transform.translation.x < 0.0 {
                            scoreboard.1 += 1;
                        }
                        text.value = format!("{} {}", scoreboard.0, scoreboard.1);
                        *velocity = Vec3::new(0.0, 0.0, 0.0);
                        ball_transform.translation = Vec3::new(0.0, 0.0, 0.0);
                        ball_reset_timer.1 = true;
                        ball_reset_timer.0.reset();
                    }
                }

                match collision {
                    Collision::Left => {
                        if velocity.x > 0.0 {
                            velocity.x = -velocity.x;
                            ball_transform.translation.x = other_transform.translation.x
                                - other_sprite.size.x / 2.0
                                - ball_size.x / 2.0;
                        }
                    }
                    Collision::Right => {
                        if velocity.x < 0.0 {
                            velocity.x = -velocity.x;
                            ball_transform.translation.x = other_transform.translation.x
                                + other_sprite.size.x / 2.0
                                + ball_size.x / 2.0;
                        }
                    }
                    Collision::Top => {
                        if velocity.y < 0.0 {
                            velocity.y = -velocity.y;
                            ball_transform.translation.y = other_transform.translation.y
                                + other_sprite.size.y / 2.0
                                + ball_size.y / 2.0;
                        }
                    }
                    Collision::Bottom => {
                        if velocity.y > 0.0 {
                            velocity.y = -velocity.y;
                            ball_transform.translation.y = other_transform.translation.y
                                - other_sprite.size.y / 2.0
                                - ball_size.y / 2.0;
                        }
                    }
                }

                break;
            }
        }
    }
}
