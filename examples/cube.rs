use bevy::prelude::*;
use bevy_fbx::FbxPlugin;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "simple cube".into(),

        ..default()
    })
    .insert_resource(bevy::log::LogSettings {
        level: bevy::log::Level::DEBUG,
        filter: "wgpu=warn,bevy_ecs=info,naga=info,gilrs=info,bevy_fbx=trace".to_string(),
    });

    app.add_plugins(DefaultPlugins);
    app.add_plugin(FbxPlugin);

    app.add_system(setup);

    app.run();
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let cube: Handle<Scene> = asset_server.load("cube.fbx#Scene");

    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 3.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    cmd.spawn_bundle(camera);

    cmd.spawn_bundle(TransformBundle {
        local: Transform::from_xyz(0.0, 0.0, 0.0),
        global: GlobalTransform::identity(),
    })
    .with_children(|parent| {
        parent.spawn_scene(cube);
    });
}
