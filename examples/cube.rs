use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
    window::{close_on_esc, WindowResolution},
};
use bevy_mod_fbx::FbxPlugin;

#[derive(Component)]
pub struct Spin;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::INFO,
                filter: "bevy_mod_fbx=trace,wgpu=warn".to_owned(),
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Spinning Cube".into(),
                    resolution: WindowResolution::new(756., 574.),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugin(FbxPlugin)
    .add_startup_system(setup)
    .add_system(spin_cube)
    .add_system(close_on_esc);

    app.run();
}

fn spin_cube(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.3 * time.delta_seconds());
        transform.rotate_local_x(0.3 * time.delta_seconds());
        transform.rotate_local_z(0.3 * time.delta_seconds());
    }
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    // Orthographic camera
    cmd.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // light
    cmd.spawn(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });

    // Cube
    cmd.spawn((
        SceneBundle {
            scene: asset_server.load("cube.fbx#Scene"),
            ..default()
        },
        Spin,
    ));
}
