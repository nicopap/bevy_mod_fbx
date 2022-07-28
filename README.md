# bevy_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

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
- [ ] Proper handling of Coordinate system
- [ ] Support `bevy_animation` as optional feature ([#13](https://github.com/HeavyRain266/bevy_fbx/issues/13))
- [ ] Provide examples with usage of complex scenes ([#6](https://github.com/HeavyRain266/bevy_fbx/issues/6))
- [ ] Convert lambert into PBR and load materials ([#12](https://github.com/HeavyRain266/bevy_fbx/issues/12))
- [ ] Expand/rewrite triangulator ([#11](https://github.com/HeavyRain266/bevy_fbx/issues/11))

### Examples

- cube: Load simple cube with point light
- scene_viwer: Load any FBX files from `/path/to/file.fbx`, defaults to `assets/cube.fbx`

Run example:

```sh
# Regular dev build
cargo run --example <example_name>

# Faster asset loading
cargu run --example <example_name> --release --features bevy/dynamic
```

## License

bevy_fbx is licensed under either:

- Apache License 2.0
- MIT License

at your option.
