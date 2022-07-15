use bevy::{prelude::*, render::camera::ScalingMode, window::close_on_esc};
use bevy_fbx::FbxPlugin;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "simple cube".into(),
        width: 756.0,
        height: 574.0,

        ..default()
    });

    app.add_plugins(DefaultPlugins);
    app.add_plugin(FbxPlugin);

    app.add_startup_system(setup);
    app.add_system(close_on_esc);

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
    cmd.spawn_bundle(TransformBundle {
        local: Transform::from_xyz(0.0, 0.0, 0.0),
        global: GlobalTransform::identity(),
    })
    .with_children(|parent| {
        parent.spawn_bundle(SceneBundle {
            scene: cube,

            ..default()
        });
    });
}
