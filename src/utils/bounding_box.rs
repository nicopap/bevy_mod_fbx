//! Bounding box.
// Derived frmo https://github.com/lo48576/fbx-viewer/blob/develop/src/util/bbox.rs

#![allow(unused)] // Remove it later.

use bevy_math::Vec3;
use std::iter::FromIterator;

/// 3D bounding box.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox3d {
    /// Minimum.
    min: Vec3,
    /// Maximum.
    max: Vec3,
}

impl BoundingBox3d {
    /// Returns minimum xyz.
    pub fn min(&self) -> Vec3 {
        self.min
    }

    /// Returns maximum xyz.
    pub fn max(&self) -> Vec3 {
        self.max
    }

    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Extedns the bounding box to contain the given point.
    pub fn insert(&self, p: Vec3) -> Self {
        Self {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    /// Extedns the bounding box to contain the given points.
    pub fn insert_extend(&self, iter: impl IntoIterator<Item = Vec3>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.insert(p))
    }

    /// Merges the bounding boxes.
    pub fn union(&self, o: &BoundingBox3d) -> Self {
        Self {
            min: self.min.min(o.min),
            max: self.max.max(o.max),
        }
    }

    /// Merges the bounding boxes.
    pub fn union_extend(&self, iter: impl IntoIterator<Item = BoundingBox3d>) -> Self {
        iter.into_iter().fold(*self, |bbox, o| bbox.union(&o))
    }
}

impl From<Vec3> for BoundingBox3d {
    fn from(p: Vec3) -> Self {
        Self { min: p, max: p }
    }
}

impl From<&Vec3> for BoundingBox3d {
    fn from(p: &Vec3) -> Self {
        Self { min: *p, max: *p }
    }
}

/// 3D bounding box, which can be empty.
#[derive(Debug, Default, Clone, Copy)]
pub struct OptionalBoundingBox3d {
    /// Bounding box.
    bbox: Option<BoundingBox3d>,
}

impl OptionalBoundingBox3d {
    /// Creates a new `OptionalBoundingBox3d`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the bounding box.
    pub fn bounding_box(&self) -> Option<BoundingBox3d> {
        self.bbox
    }

    /// Extedns the bounding box to contain the given point.
    pub fn insert(&self, p: Vec3) -> Self {
        self.bbox
            .map_or_else(|| p.into(), |bbox| bbox.insert(p))
            .into()
    }

    /// Extedns the bounding box to contain the given points.
    pub fn insert_extend(&self, iter: impl IntoIterator<Item = Vec3>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.insert(p))
    }

    /// Merges the bounding boxes.
    pub fn union(&self, o: &OptionalBoundingBox3d) -> Self {
        match (&self.bbox, &o.bbox) {
            (Some(b), Some(o)) => b.union(o).into(),
            (Some(v), None) | (None, Some(v)) => v.into(),
            (None, None) => Self::new(),
        }
    }

    /// Merges the bounding boxes.
    pub fn union_extend(&self, iter: impl IntoIterator<Item = OptionalBoundingBox3d>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.union(&p))
    }
}

impl From<BoundingBox3d> for OptionalBoundingBox3d {
    fn from(bbox: BoundingBox3d) -> Self {
        Self { bbox: Some(bbox) }
    }
}

impl From<&BoundingBox3d> for OptionalBoundingBox3d {
    fn from(bbox: &BoundingBox3d) -> Self {
        Self { bbox: Some(*bbox) }
    }
}

impl From<Option<BoundingBox3d>> for OptionalBoundingBox3d {
    fn from(bbox: Option<BoundingBox3d>) -> Self {
        Self { bbox }
    }
}

impl From<Vec3> for OptionalBoundingBox3d {
    fn from(p: Vec3) -> Self {
        BoundingBox3d::from(p).into()
    }
}

impl From<&Vec3> for OptionalBoundingBox3d {
    fn from(p: &Vec3) -> Self {
        BoundingBox3d::from(*p).into()
    }
}

impl From<Option<Vec3>> for OptionalBoundingBox3d {
    fn from(p: Option<Vec3>) -> Self {
        Self {
            bbox: p.map(BoundingBox3d::from),
        }
    }
}

impl From<Option<&Vec3>> for OptionalBoundingBox3d {
    fn from(p: Option<&Vec3>) -> Self {
        Self {
            bbox: p.map(BoundingBox3d::from),
        }
    }
}

impl FromIterator<Vec3> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Vec3>,
    {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(v) => v,
            None => return Self::default(),
        };
        Self {
            bbox: Some(BoundingBox3d::from(first).insert_extend(iter)),
        }
    }
}

impl<'a> FromIterator<&'a Vec3> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a Vec3>,
    {
        iter.into_iter().copied().collect()
    }
}

impl FromIterator<BoundingBox3d> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = BoundingBox3d>,
    {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(v) => v,
            None => return Self::default(),
        };
        Self {
            bbox: Some(first.union_extend(iter)),
        }
    }
}

impl<'a> FromIterator<&'a BoundingBox3d> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a BoundingBox3d>,
    {
        iter.into_iter().copied().collect()
    }
}

impl FromIterator<OptionalBoundingBox3d> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = OptionalBoundingBox3d>,
    {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(v) => v,
            None => return Self::default(),
        };
        first.union_extend(iter)
    }
}

impl<'a> FromIterator<&'a OptionalBoundingBox3d> for OptionalBoundingBox3d {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a OptionalBoundingBox3d>,
    {
        iter.into_iter().copied().collect()
    }
}
