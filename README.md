# bevy_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

### Cargo features

#### `profile`

Enables spans,
in combination with bevy's `bevy/trace` feature,
you can generate profiling reports you can open with any trace reading software.
Useful for debugging why your assets are so slow to load.

#### `maya_3dsmax_pbr`

Enable handling of Maya's PBR material extension for FBX (presumebly also 3DS max).
This is highly experimental and only tested with a single model!
Please report if your model's materials do not load properly.

This material loader do not work with every type of texture files,
the textures must be readable from CPU and have each component (color channel)
be exactly 8 bits (such as PNG).

### Version matrix

| bevy | latest supporting version      |
|------|--------|
| 0.8  | 0.1.0-dev |

### Limitations

> **NOTE**: If you find more limitations, file an issue!

- Only binary version of FBX 7.4 & 7.5 are supported.
- No ASCII FBX format support is planned. (blocked by upstream)
- No support for multiple scenes within single FBX file.

### What have to be done

- [X] Replace `cgmath` with `bevy_math`
- [X] Write basic loader
- [X] WebAssembly support
- [X] Load textures
- [X] Support arbitrary material loading.
- [X] Support Maya's PBR
- [X] Proper scaling based on FBX config scale ([#10](https://github.com/HeavyRain266/bevy_fbx/issues/10))
- [X] Load complex scenes with a transform tree hierarchy
- [ ] Proper handling of Coordinate system
- [ ] Support `bevy_animation` as optional feature ([#13](https://github.com/HeavyRain266/bevy_fbx/issues/13))
- [ ] Provide examples with usage of complex scenes ([#6](https://github.com/HeavyRain266/bevy_fbx/issues/6))
- [ ] Convert lambert into PBR and load materials ([#12](https://github.com/HeavyRain266/bevy_fbx/issues/12))
- [ ] Expand/rewrite triangulator ([#11](https://github.com/HeavyRain266/bevy_fbx/issues/11))

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

### Contributing

FBX is a very widely used and flexible file format.
We currently expect most models **to not load properly**.
We cannot test how `bevy_fbx` handles the export formats of every software out there.
If your model loads properly, thank your lucky start,
if you encounter any issue please do the following:

- Try opening the model using the `scene_viewer` (run `cargo run --example scene_viewer --release -- /path/to/file.fbx`)
  (if your file has textures, make sure to enable the correct file formats using `--features bevy/png bevy/jpg bevy/tga` etc.)
- If it fails, open an [issue] on our Github repo with a screenshot in the scene viewer
  and a screenshot of how the model should look like
- Ideally provide a download link to your FBX model

#### Further troubleshooting tools

If you want to help us figure out how to load a particularly tricky model,
the following tools may be useful:

- <https://github.com/lo48576/fbx_objects_depviz>
- <https://github.com/lo48576/fbx-tree-view>
- Use the `profile` feature with the [bevy profiling instructions]

#### Licensing

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed
as in the [License](#license) section,
without any additional terms or conditions.

## License

Original loader and triangulation code (`loader.rs` and `triangulate.rs`)
from [fbx viewer] by YOSHIOKA Takuma.
Original scene viewer code (`scene_viewer.rs`)
from [bevy scene viewer] by bevy contributors.
All additions and modifications authored by `bevy_fbx` contributors (see git log).

Code copyrights go to their respective authors.

All code in `bevy_fbx` is licensed under either:

- Apache License 2.0
- MIT License

at your option.

[fbx viewer]: https://github.com/lo48576/fbx-viewer/
[issue]: https://github.com/HeavyRain266/bevy_fbx/issues/new/choose
[bevy profiling instructions]: https://github.com/bevyengine/bevy/blob/main/docs/profiling.md
[bevy scene viewer]: https://github.com/bevyengine/bevy/blob/115211161b783a2f5c39346caeb8ee6b3b202bef/examples/tools/scene_viewer.rs
