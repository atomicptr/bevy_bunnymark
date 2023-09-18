use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResized};
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};
use rand::Rng;

const BUNNY_WIDTH: f32 = 26.0;
const BUNNY_HEIGHT: f32 = 37.0;
const TILE_SIZE: u32 = 32;
const DEFAULT_BUNNIES: u64 = 128;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Bunny Mark".into(),
                        resizable: true,
                        ..default()
                    }),
                    exit_condition: ExitCondition::OnPrimaryClosed,
                    close_when_requested: true,
                })
                .build(),
        )
        .add_plugins(PixelCameraPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                resize_background,
                spawn_bunnies,
                bunny_movement,
                bunny_spawn_controller,
                update_bunny_text,
            ),
        )
        .insert_resource(BunnyCount {
            current: 0,
            desired: DEFAULT_BUNNIES,
        })
        .run();
}

#[derive(Component)]
struct Bunny {
    direction: Vec3,
    speed_factor: f32,
}

#[derive(Component)]
struct BunnyParent;

#[derive(Component)]
struct BunnyText;

#[derive(Resource)]
struct BunnyCount {
    current: u64,
    desired: u64,
}

#[derive(Component)]
struct Grass(i32, i32);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, window: Query<&Window>) {
    commands.spawn(PixelCameraBundle::from_zoom(1));

    let window = window.single();

    render_background(
        window.resolution.width(),
        window.resolution.height(),
        &mut commands,
        &asset_server,
        None,
    );

    commands.spawn((
        SpatialBundle::default(),
        BunnyParent,
        Name::new("Bunny Parent"),
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("UI Root"),
        ))
        .with_children(|commands| {
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Num. Bunnies: ...",
                        TextStyle {
                            font_size: 16.0,
                            ..default()
                        },
                    ),
                    ..default()
                },
                BunnyText,
            ));
        });
}

fn resize_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<WindowResized>,
    grass: Query<&Grass>,
) {
    for event in events.iter() {
        render_background(
            event.width,
            event.height,
            &mut commands,
            &asset_server,
            Some(&grass),
        );
    }
}

fn render_background(
    width: f32,
    height: f32,
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    grass: Option<&Query<&Grass>>,
) {
    let tiles_width = ((width / TILE_SIZE as f32 + 1.0) * 0.5).ceil() as i32;
    let tiles_height = ((height / TILE_SIZE as f32 + 1.0) * 0.5).ceil() as i32;

    for i in -tiles_width..tiles_width {
        for j in -tiles_height..tiles_height {
            if let Some(grass) = grass {
                let mut has_tile = false;

                for grass in grass {
                    if i == grass.0 && j == grass.1 {
                        has_tile = true;
                        break;
                    }
                }

                if has_tile {
                    continue;
                }
            }

            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("grass.png"),
                    transform: Transform::from_xyz(
                        i as f32 * TILE_SIZE as f32,
                        j as f32 * TILE_SIZE as f32,
                        -1.0,
                    ),
                    ..default()
                },
                Grass(i, j),
            ));
        }
    }
}

fn spawn_bunnies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    parent: Query<Entity, With<BunnyParent>>,
    mut bunny_count: ResMut<BunnyCount>,
) {
    let parent = parent.single();

    let bunnies_to_draw = bunny_count.desired - bunny_count.current;

    if bunnies_to_draw <= 0 {
        return;
    }

    let mut rng = rand::thread_rng();

    commands.entity(parent).with_children(|commands| {
        for _ in 0..bunnies_to_draw {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("wabbit_alpha.png"),
                    ..default()
                },
                Bunny {
                    direction: Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0)
                        .normalize(),
                    speed_factor: rng.gen_range(1.0..4.0),
                },
                Name::new("Bunny"),
            ));

            bunny_count.current += 1;
        }
    });
}

fn bunny_movement(
    mut bunnies: Query<(&mut Transform, &mut Bunny)>,
    time: Res<Time>,
    window: Query<&Window>,
) {
    let window = window.single();

    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    for (mut transform, mut bunny) in &mut bunnies {
        let speed = 200.0 * bunny.speed_factor * time.delta_seconds();

        let half_width = window_width as f32 * 0.5 - BUNNY_WIDTH * 0.5;
        let half_height = window_height as f32 * 0.5 - BUNNY_HEIGHT * 0.5;

        if transform.translation.x <= -half_width || transform.translation.x >= half_width {
            bunny.direction.x *= -1.0;
        }

        if transform.translation.y <= -half_height || transform.translation.y >= half_height {
            bunny.direction.y *= -1.0;
        }

        transform.translation += bunny.direction * speed;

        if transform.translation.x < (window_width as f32 * -2.0)
            || transform.translation.x > (window_width as f32 * 2.0)
            || transform.translation.y < (window_height as f32 * -2.0)
            || transform.translation.y > (window_height as f32 * 2.0)
        {
            transform.translation = Vec3::default();
        }
    }
}

fn bunny_spawn_controller(input: Res<Input<KeyCode>>, mut bunny_count: ResMut<BunnyCount>) {
    if bunny_count.current != bunny_count.desired {
        return;
    }

    if !input.just_pressed(KeyCode::Space) {
        return;
    }

    bunny_count.desired *= 2;
}

fn update_bunny_text(mut texts: Query<&mut Text, With<BunnyText>>, bunny_count: Res<BunnyCount>) {
    for mut text in &mut texts {
        text.sections[0].value = format!("Num. Bunnies: {}", bunny_count.current)
    }
}
