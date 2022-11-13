# bevy_mod_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

**Special Credit**: Thanks to the original author HeavyRain266 for starting the project.
`bevy_mod_fbx` is now maintained by someone else.

### Features

- Load meshes, textures & material properties
- Supported material properties:
  - normal maps
  - occlusion maps
  - diffuse texture
- Maya PBR material support
- Scene tree transform hierarchy support

#### Planned features

- Skeleton rig imports
- `bevy_animation` support
- Optional lambert material shader support
- Optional phong shading model support
- Extended compatibility:
  - `IndexToDirect`
  - Handle file-based axis properties
  - Handle backed cameras & lights
  - N-gon triangulation

### Limitations

- FBX v7.4 & 7.5 are the only supported versions
- FBX doesn't support multiple scenes in single file, use multiple files instead
- There are no plans for loading ASCII format, export FBX as binary v7.4/7.5
- There is no support for complex shapes at the moment, see [#11]

### Cargo features

#### `profile`

Enables spans, in combination with bevy's `bevy/trace` feature,
you can generate profiling reports you can open with any trace reading software.
Useful for debugging why your assets are so slow to load.

#### `maya_3dsmax_pbr`

Enable handling of Maya's PBR material extension for FBX (presumebly also 3DS max).
This is highly experimental and only tested with a single model!
Please report if your model's materials do not load properly.

This material loader do not work with every type of texture files,
the textures must be readable from CPU and have each component (color channel)
be exactly 8 bits (such as PNG).

### Examples

- `cube`: Load simple cube with point light
- `scene_viewer`: Load any FBX files from `/path/to/file.fbx`, defaults to `assets/cube.fbx`

Run example:

```sh
# Regular dev build
cargo run --example <example_name>

# Faster asset loading
cargu run --example <example_name> --release --features bevy/dynamic
```

### Version matrix

| bevy | bevy_mod_fbx |
|------|--------------|
| 0.9  | 0.2          |
| 0.8  | 0.1.0-dev    |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed informations.

## License

Original loader and triangulation code (`loader.rs` and `triangulate.rs`) from [fbx_viewer] by YOSHIOKA Takuma.
Original scene viewer code (`scene_viewer.rs`) from [scene_viewer] by Bevy contributors.
All additions and modifications authored by `bevy_mod_fbx` contributors (see git log).

Code copyrights go to their respective authors.

All code in `bevy_mod_fbx` is licensed under either:

- Apache License 2.0
- MIT License

at your option.

[#11]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/11

[fbx_viewer]: https://github.com/lo48576/fbx-viewer/
[bevy_scene_viewer]: https://github.com/bevyengine/bevy/blob/115211161b783a2f5c39346caeb8ee6b3b202bef/examples/tools/scene_viewer.rs
