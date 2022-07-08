//! Texture.

use image::DynamicImage;

/// Texture.
#[derive(Clone)]
pub struct Texture {
    /// Name.
    pub name: Option<String>,
    /// Image.
    pub image: DynamicImage,
    /// Whether the texture can be transparent.
    ///
    /// If `false`, the texture can be assumed to have no transparent texels.
    pub transparent: bool,
    /// Wrap mode for U axis.
    pub wrap_mode_u: WrapMode,
    /// Wrap mode for V axis.
    pub wrap_mode_v: WrapMode,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        /// Image info.
        #[derive(Debug)]
        struct ImageInfo {
            /// Width.
            width: u32,
            /// Height.
            height: u32,
            /// Color type.
            color: image::ColorType,
        }

        f.debug_struct("Texture")
            .field("name", &self.name)
            .field(
                "image",
                &ImageInfo {
                    width: self.image.width(),
                    height: self.image.height(),
                    color: self.image.color(),
                },
            )
            .field("transparent", &self.transparent)
            .field("wrap_mode_u", &self.wrap_mode_u)
            .field("wrap_mode_v", &self.wrap_mode_v)
            .finish()
    }
}

/// Wrap mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WrapMode {
    /// Repeat.
    Repeat,
    /// Clamp to edge.
    ClampToEdge,
}
