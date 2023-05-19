use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
    window::{close_on_esc, WindowResolution},
};
use bevy_mod_fbx::{FbxPlugin, FbxScene};

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
    .add_systems(Startup, setup)
    .add_systems(Update, (spin_cube, close_on_esc, print_fbx));

    app.run();
}

fn spin_cube(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.3 * time.delta_seconds());
        transform.rotate_local_x(0.3 * time.delta_seconds());
        transform.rotate_local_z(0.3 * time.delta_seconds());
    }
}

#[derive(Resource)]
struct StoreAssets(Handle<FbxScene>);

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

    cmd.insert_resource(StoreAssets(asset_server.load("cube.fbx#FbxScene")));
    // Cube
    cmd.spawn((
        SceneBundle {
            scene: asset_server.load("cube.fbx#Scene"),
            ..default()
        },
        Spin,
    ));
}
fn print_fbx(
    key_input: Res<Input<KeyCode>>,
    scenes: Res<Assets<FbxScene>>,
    b_scenes: Res<Assets<Scene>>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    mats: Res<Assets<StandardMaterial>>,
    names: Query<(DebugName, Option<&Visibility>, Option<&Children>)>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        println!("FbxScene");
        for scene in scenes.iter() {
            println!("{scene:?}");
        }
        println!("Scene");
        for scene in b_scenes.iter() {
            println!("{scene:?}");
        }
        println!("Image");
        for (image, _) in images.iter() {
            println!("{image:?}");
        }
        println!("Mesh");
        for (mesh, _) in meshes.iter() {
            println!("{mesh:?}");
        }
        println!("StandardMaterial");
        for (mat, mat_value) in mats.iter() {
            println!("{mat:?} {mat_value:?}");
        }
        println!("DebugName");
        for (name, vis, ch) in &names {
            println!("{name:?} {vis:?} {ch:?}");
        }
    }
}
