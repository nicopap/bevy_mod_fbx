## bevy_fbx

Autodesk Filmbox (*.fbx) loader for Bevy Engine.

> **INFO**: All the base code is derived from [fbx-viewer] by the author of [fbxcel] & [fbxcel-dom] libraries.

### Limitations

> **NOTE**: If you find more limitations, file an issue!

- Only binary version of FBX 7400 will be supported.
- No ASCII FBX format support is planned. (blocked by upstream)

### What have to be done

- [X] Replace `cgmath` with `bevy_math`
- [ ] Write basic loader
- [ ] Add support for `bevy_animations`
- [ ] Provide examples with usage of complex scenes
- [ ] Possibly test cutscenes loaded with this plugin

### Develpment tools

uuidgen - generate derive macro with alredy applied UUID.

```sh
gem install securerandom

ruby tools/uuidgen.rb # or chmod +x tools/uuidgen.rb; ./tools/uuidgen.rb
```

check_fbx - parse FBX files and print some useful informations about.

```sh
cd tools/check_fbx

cargo run /path/to/file.fbx
```

## License

bevy_fbx is licensed under either:

- Apache License 2.0
- MIT License

at your option.

[fbxcel]: https://github.com/lo48576/fbxcel/
[fbxcel-dom]: https://github.com/lo48576/fbxcel-dom/
[fbx-viewer]: https://github.com/lo48576/fbx-viewer/
