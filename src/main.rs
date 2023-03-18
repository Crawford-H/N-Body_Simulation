use bevy::{
    prelude::*, 
    sprite::MaterialMesh2dBundle,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(move_square)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}

#[derive(Component)]
struct Square;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct FPS;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        ..default()
        },
        Square,
    ));
}

fn move_square(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Square>>) {
    let mut square_transform = query.single_mut();
    let mut x_direction = 0.0;
    let mut y_direction = 0.0;

    if keyboard_input.pressed(KeyCode::W) {
        y_direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        y_direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) {
        x_direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        x_direction += 1.0;
    }

    square_transform.translation.x += x_direction;
    square_transform.translation.y += y_direction;
    square_transform.rotate_z(0.01);
}