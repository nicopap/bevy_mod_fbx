//! Material.

use rgb::RGB;

/// Material.
#[derive(Debug, Clone)]
pub struct Material {
    /// Name.
    pub name: Option<String>,
    /// Shading parameters.
    pub data: ShadingData,
}

/// Shading data.
#[derive(Debug, Clone, Copy)]
pub enum ShadingData {
    /// Lambert material.
    Lambert(LambertData),
}

/// Lambert data.
#[derive(Debug, Clone, Copy)]
pub struct LambertData {
    /// Ambient.
    pub ambient: RGB<f32>,
    /// Diffuse.
    pub diffuse: RGB<f32>,
    /// Emissive.
    pub emissive: RGB<f32>,
}
