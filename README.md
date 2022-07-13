## bevy_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

### Limitations

> **NOTE**: If you find more limitations, file an issue!

- Only binary version of FBX 7.4 & 7.5 are supported.
- No ASCII FBX format support is planned. (blocked by upstream)

### What have to be done

- [X] Replace `cgmath` with `bevy_math`
- [X] Write basic loader
- [X] Load textures
- [ ] Convert lambert into PBR and load materials
- [ ] Multiple scenes and entities
- [ ] Add support for `bevy_animation`
- [ ] Provide examples with usage of complex scenes

### Examples

Load any FBX files to render them into bevy, tool code derived from the bevy
native gltf loader.

```sh
cargo run --example scene_viewer path/to/file.fbx#Scene

# Faster load time and compile time
cargo run --example scene_viewer --features "bevy/dynamic" --release path/to/file.fbx#Scene
```

Load `cube.fbx` with orthographics projection
```sh
cargo run --example cube

# Faster load time and compile time
cargo run --example cube --features "bevy/dynamic" --release
```

### Develpment tools

Generate derive macro with alredy applied UUID.

```sh
gem install securerandom

ruby tools/uuidgen.rb # or chmod +x tools/uuidgen.rb; ./tools/uuidgen.rb
```

## License

bevy_fbx is licensed under either:

- Apache License 2.0
- MIT License

at your option.
