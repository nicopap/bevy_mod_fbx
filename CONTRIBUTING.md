# Contributing

### About

FBX is a very widely used and flexible file format.
We currently expect most models **to not load properly**.
We cannot test how `bevy_mod_fbx` handles the export formats of every software out there.
If your model loads properly, thank your lucky start,
if you encounter any issue please do the following:

- Try opening the model using the `scene_viewer` (run `cargo run --example scene_viewer --release -- /path/to/file.fbx`)
  (if your file has textures, make sure to enable the correct file formats using `--features bevy/png bevy/jpg bevy/tga` etc.)
- If it fails, open an [issue] on our Github repo with a screenshot in the scene viewer
  and a screenshot of how the model should look like
- Ideally provide a download link to your FBX model

#### What have to be done

- [ ] Proper handling of Coordinate system
- [ ] Support `bevy_animation` as optional feature ([#13])
- [ ] Provide examples with usage of complex scenes ([#6])
- [ ] Convert lambert into PBR and load materials ([#12])
- [ ] Expand/rewrite triangulator ([#11])

#### Commiting changes

Before you commit any changes, ensure that `rustfmt` is installed and then run `cargo fmt`.

#### Further troubleshooting tools

If you want to help us figure out how to load a particularly tricky model,
the following tools may be useful:

- <https://github.com/lo48576/fbx_objects_depviz>
- <https://github.com/lo48576/fbx-tree-view>
- Use the `profile` feature with the [profiling] instructions from Bevy Engine

### Licensing

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed
as in the [License](#license) section,
without any additional terms or conditions.

[issue]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/new/choose
[profiling]: https://github.com/bevyengine/bevy/blob/main/docs/profiling.md

[#13]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/13
[#6]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/6
[#12]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/12
[#11]: https://github.com/HeavyRain266/bevy_mod_fbx/issues/11
