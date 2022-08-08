# Custom materials

FBX is a widespread standard.
Each individual software that works with FBX has their own material definition,
implemented as custom extensions to the basic FBX model.

It is likely that the library user will want to load a material type that we,
as library developers, did not or couldn't anticipate.

To still allow them to load their material types, we define an API
for the user to provide their own material loader.

## API requirements

* Provide default loaders for lambert/phong
* Provide loader attempt order, so that the first to "win" is the one loaded.
  Loader should be able to say "I can't load that", to let other loaders try
* Ability to load assets like `Image`s
  * _Direct access to `Image`_ (not just `Handle<Image>`) so that they
    can be combined (in the case of Maya's PBR, reflection and metalicness
    maps are two different textures, they need to be combined for bevy's PBR)
  * This requires ability to register `Handle`s
    (maybe? Could we provide an untyped `Handle` registration API?)
* Should be able to separate async operations from pure operations
  For example, the loader could give a list of asset paths to load
  and a function that takes the deserialized typed assets from those paths
  and return a new asset.
  (because `async` fundamentally composes poorly with combinators like
  `Iterator` or `Option` and `Result`)
* Loader should be able to take a single `MaterialHandle` from `fbxcell_dom` lib
  and return a sort of material component.
* (optional) Ability to load other types of assets
* (optional) Ability to return more than just bevy's `StandardMaterial` PBR shader
* (optional) Provide loaders for FBX extensions like Maya's PBR

## How to solve this

Ideally, bevy provides a **composable loader API**.

However, let's point out there are two types of asset composition:
* Taking multiple assets and combining them into one,
  such as with the reflection/metalicness map example from earlier.
  * I guess we could just create a new component that creates the new
    texture based on the two old ones after loading and replace them
    with the combined version?
* Taking handles to existing assets and using them in new assets.

But we are working with today's bevy, so we have to go with a bespoke solution.

### Design

See https://github.com/HeavyRain266/bevy_mod_fbx/issues/18

Maybe we should _delegate_ fbx material loading to a different `AssetLoader`?
How would that work? I'd like to be able to give it a `MaterialHandle`,
not a file path or a `Vec<u8>`.

#### Proper handling of `Handle`s

**Problem**: We want to cache textures with a globally identifying name,
so that it's possible to re-use the same handle instead of loading it
again.
But lose the exact mapping of fbxcel `TextureHandle` to label after running
`preprocess_textures`.
How to reconciliate?

- If we pre-process, we lose the 1-to-1 mapping between `TextureHandle` and
  bevy `Handle<Image>`.
- We generally want to "store" the `Image`s as they are output by the
  `preprocess_textures` function.
- The caching requires 
  - **being executed before the image is loaded**
  - a `&mut` access to the `scene` in the loader
  - a `&mut` access to the `LoadContext` to create the asset

**Solution**: Split the list of loaded textures in two.

The resulting code is still very awkward, but it's still better than before.

This is hopeless anyway (well it depends) because merging two different textures
requires being able to read and edit the texture format, which is far from trivial
in bevy.