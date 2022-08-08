use bevy::{
    log::{Level, LogSettings},
    prelude::*,
    render::camera::ScalingMode,
    window::close_on_esc,
};
use bevy_mod_fbx::FbxPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "simple cube".into(),
        width: 756.0,
        height: 574.0,

        ..default()
    })
    .insert_resource(LogSettings {
        level: Level::INFO,
        filter: "bevy_mod_fbx=trace,wgpu=warn".to_owned(),
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(WorldInspectorPlugin::new())
    .add_plugin(FbxPlugin)
    .add_startup_system(setup)
    .add_system(close_on_esc);

    app.run();
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let cube: Handle<Scene> = asset_server.load("cube.fbx#Scene");

    // Orthographic camera
    cmd.spawn_bundle(Camera3dBundle {
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
    cmd.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });

    // Cube
    cmd.spawn_bundle(SceneBundle {
        scene: cube,
        ..default()
    });
}
