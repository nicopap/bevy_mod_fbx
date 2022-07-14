# bevy_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

### Limitations

> **NOTE**: If you find more limitations, file an issue!

- Only binary version of FBX 7.4 & 7.5 are supported.
- No ASCII FBX format support is planned. (blocked by upstream)

### What have to be done

- [X] Replace `cgmath` with `bevy_math`
- [X] Write basic loader
- [X] WebAssembly support
- [X] Load textures
- [ ] Convert lambert into PBR and load materials
- [ ] Multiple scenes and entities
- [ ] Support `bevy_animation` as optional feature
- [ ] Provide examples with usage of complex scenes

### Examples

- cube: Load simple cube with point light
- scene_viwer: Load any FBX files from `/path/to/file.fbx`, defaults to `assets/cube.fbx`

Run example:

```sh
# Regular dev build
cargo run --example <example_name>

# Faster asset loading
cargu run --example <example_name> --release --features="bevy/dynamic"
```

## License

bevy_fbx is licensed under either:

- Apache License 2.0
- MIT License

at your option.
