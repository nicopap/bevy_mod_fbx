//! Bounding box.
// Derived frmo https://github.com/lo48576/fbx-viewer/blob/develop/src/util/bbox.rs

use std::iter::FromIterator;

use cgmath::{num_traits::Float, BaseFloat, Point3, Vector3};

/// 3D bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox3d<S> {
    /// Minimum.
    min: Point3<S>,
    /// Maximum.
    max: Point3<S>,
}

impl<S: BaseFloat> BoundingBox3d<S> {
    /// Returns minimum xyz.
    pub fn min(&self) -> Point3<S> {
        self.min
    }

    /// Returns maximum xyz.
    pub fn max(&self) -> Point3<S> {
        self.max
    }

    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vector3<S> {
        use cgmath::EuclideanSpace;

        self.max.to_vec() - self.min.to_vec()
    }

    /// Extedns the bounding box to contain the given point.
    pub fn insert(&self, p: Point3<S>) -> Self {
        Self {
            min: element_wise_apply(self.min, p, Float::min),
            max: element_wise_apply(self.max, p, Float::max),
        }
    }

    /// Extedns the bounding box to contain the given points.
    pub fn insert_extend(&self, iter: impl IntoIterator<Item = Point3<S>>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.insert(p))
    }

    /// Merges the bounding boxes.
    pub fn union(&self, o: &BoundingBox3d<S>) -> Self {
        Self {
            min: element_wise_apply(self.min, o.min, Float::min),
            max: element_wise_apply(self.max, o.max, Float::max),
        }
    }

    /// Merges the bounding boxes.
    pub fn union_extend(&self, iter: impl IntoIterator<Item = BoundingBox3d<S>>) -> Self {
        iter.into_iter().fold(*self, |bbox, o| bbox.union(&o))
    }
}

impl<S: BaseFloat> From<Point3<S>> for BoundingBox3d<S> {
    fn from(p: Point3<S>) -> Self {
        Self { min: p, max: p }
    }
}

impl<S: BaseFloat> From<&Point3<S>> for BoundingBox3d<S> {
    fn from(p: &Point3<S>) -> Self {
        Self { min: *p, max: *p }
    }
}

/// 3D bounding box, which can be empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalBoundingBox3d<S> {
    /// Bounding box.
    bbox: Option<BoundingBox3d<S>>,
}

impl<S: BaseFloat> OptionalBoundingBox3d<S> {
    /// Creates a new `OptionalBoundingBox3d`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the bounding box.
    pub fn bounding_box(&self) -> Option<BoundingBox3d<S>> {
        self.bbox
    }

    /// Extedns the bounding box to contain the given point.
    pub fn insert(&self, p: Point3<S>) -> Self {
        self.bbox
            .map_or_else(|| p.into(), |bbox| bbox.insert(p))
            .into()
    }

    /// Extedns the bounding box to contain the given points.
    pub fn insert_extend(&self, iter: impl IntoIterator<Item = Point3<S>>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.insert(p))
    }

    /// Merges the bounding boxes.
    pub fn union(&self, o: &OptionalBoundingBox3d<S>) -> Self {
        match (&self.bbox, &o.bbox) {
            (Some(b), Some(o)) => b.union(o).into(),
            (Some(v), None) | (None, Some(v)) => v.into(),
            (None, None) => Self::new(),
        }
    }

    /// Merges the bounding boxes.
    pub fn union_extend(&self, iter: impl IntoIterator<Item = OptionalBoundingBox3d<S>>) -> Self {
        iter.into_iter().fold(*self, |bbox, p| bbox.union(&p))
    }
}

impl<S> Default for OptionalBoundingBox3d<S> {
    fn default() -> Self {
        Self { bbox: None }
    }
}

impl<S: BaseFloat> From<BoundingBox3d<S>> for OptionalBoundingBox3d<S> {
    fn from(bbox: BoundingBox3d<S>) -> Self {
        Self { bbox: Some(bbox) }
    }
}

impl<S: BaseFloat> From<&BoundingBox3d<S>> for OptionalBoundingBox3d<S> {
    fn from(bbox: &BoundingBox3d<S>) -> Self {
        Self { bbox: Some(*bbox) }
    }
}

impl<S: BaseFloat> From<Option<BoundingBox3d<S>>> for OptionalBoundingBox3d<S> {
    fn from(bbox: Option<BoundingBox3d<S>>) -> Self {
        Self { bbox }
    }
}

impl<S: BaseFloat> From<Point3<S>> for OptionalBoundingBox3d<S> {
    fn from(p: Point3<S>) -> Self {
        BoundingBox3d::from(p).into()
    }
}

impl<S: BaseFloat> From<&Point3<S>> for OptionalBoundingBox3d<S> {
    fn from(p: &Point3<S>) -> Self {
        BoundingBox3d::from(*p).into()
    }
}

impl<S: BaseFloat> From<Option<Point3<S>>> for OptionalBoundingBox3d<S> {
    fn from(p: Option<Point3<S>>) -> Self {
        Self {
            bbox: p.map(BoundingBox3d::from),
        }
    }
}

impl<S: BaseFloat> From<Option<&Point3<S>>> for OptionalBoundingBox3d<S> {
    fn from(p: Option<&Point3<S>>) -> Self {
        Self {
            bbox: p.map(BoundingBox3d::from),
        }
    }
}

impl<S: BaseFloat> FromIterator<Point3<S>> for OptionalBoundingBox3d<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Point3<S>>,
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

impl<'a, S: 'a + BaseFloat> FromIterator<&'a Point3<S>> for OptionalBoundingBox3d<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a Point3<S>>,
    {
        iter.into_iter().copied().collect()
    }
}

impl<S: BaseFloat> FromIterator<BoundingBox3d<S>> for OptionalBoundingBox3d<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = BoundingBox3d<S>>,
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

impl<'a, S: 'a + BaseFloat> FromIterator<&'a BoundingBox3d<S>> for OptionalBoundingBox3d<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a BoundingBox3d<S>>,
    {
        iter.into_iter().copied().collect()
    }
}

impl<S: BaseFloat> FromIterator<OptionalBoundingBox3d<S>> for OptionalBoundingBox3d<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = OptionalBoundingBox3d<S>>,
    {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(v) => v,
            None => return Self::default(),
        };
        first.union_extend(iter)
    }
}

impl<'a, S: 'a + BaseFloat> FromIterator<&'a OptionalBoundingBox3d<S>>
    for OptionalBoundingBox3d<S>
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a OptionalBoundingBox3d<S>>,
    {
        iter.into_iter().copied().collect()
    }
}

/// Applies the given function element wise.
fn element_wise_apply<S, U, F>(a: Point3<S>, b: Point3<S>, f: F) -> Point3<U>
where
    F: Fn(S, S) -> U,
{
    Point3::new(f(a.x, b.x), f(a.y, b.y), f(a.z, b.z))
}
