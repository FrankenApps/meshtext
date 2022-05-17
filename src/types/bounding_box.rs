use glam::{Mat4, Vec3A};

/// A bounding box or bounding rectangle in the case of
/// a flat mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// The coordinates of the maximum point.
    ///
    /// Note that the z-coordinate will be `0f32` if
    /// the mesh is flat.
    pub max: Vec3A,

    /// The coordinates of the minimum point.
    ///
    /// Note that the z-coordinate will be `0f32` if
    /// the mesh is flat.
    pub min: Vec3A,
}

impl BoundingBox {
    /// Creates a new [BoundingBox].
    ///
    /// Arguments:
    ///
    /// * `min`: The minimum vertex of this bounding box.
    /// * `max`: The maximum vertex of this bounding box.
    ///
    /// Returns:
    ///
    /// The new [BoundingBox].
    pub fn new(min: Vec3A, max: Vec3A) -> Self {
        Self { max, min }
    }

    /// Creates a new empty [BoundingBox].
    ///
    /// Returns:
    ///
    /// The empty [BoundingBox].
    pub(crate) fn empty() -> Self {
        Self {
            max: Vec3A::ZERO,
            min: Vec3A::ZERO,
        }
    }

    /// Calculates the center of this [BoundingBox].
    ///
    /// Returns:
    ///
    /// A [Vec3A] representing the point in the geometric
    /// center of this [BoundingBox].
    ///
    /// # Example
    ///
    /// ```rust
    /// use glam::Vec3A;
    /// use meshtext::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(
    ///     Vec3A::new(0f32, 0f32, 0f32),
    ///     Vec3A::new(1f32, 1f32, 1f32),
    /// );
    ///
    /// assert_eq!(bbox.center(), Vec3A::new(0.5, 0.5, 0.5));
    /// ```
    pub fn center(&self) -> Vec3A {
        self.min + (self.max - self.min) * 0.5f32
    }

    /// Combines this and another [BoundingBox] into a new one
    /// and returns it.
    ///
    /// Arguments:
    ///
    /// * `other`: The [BoundingBox] with which this bounding box will
    /// be combined.
    ///
    /// Returns:
    ///
    /// The combined [BoundingBox].
    ///
    /// # Example
    ///
    /// ```rust
    /// use glam::Vec3A;
    /// use meshtext::BoundingBox;
    ///
    /// let bbox1 = BoundingBox::new(
    ///     Vec3A::new(0f32, 0f32, 0f32),
    ///     Vec3A::new(1f32, 1f32, 1f32),
    /// );
    ///
    /// let bbox2 = BoundingBox::new(
    ///     Vec3A::new(2f32, 2f32, 0f32),
    ///     Vec3A::new(3f32, 3f32, 1f32),
    /// );
    ///
    /// let combination = BoundingBox::new(
    ///     Vec3A::new(0f32, 0f32, 0f32),
    ///     Vec3A::new(3f32, 3f32, 1f32),
    /// );
    ///
    /// assert_eq!(bbox1.combine(&bbox2), combination);
    /// ```
    pub fn combine(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox::new(self.min.min(other.min), self.max.max(other.max))
    }

    /// Gets the size of this [BoundingBox].
    ///
    /// Returns:
    ///
    /// A [Vec3A] with the extent of this [BoundingBox]
    /// along each coordinate axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glam::Vec3A;
    /// use meshtext::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(
    ///     Vec3A::new(0f32, 0f32, 1f32),
    ///     Vec3A::new(1f32, 1f32, 3f32),
    /// );
    ///
    /// assert_eq!(bbox.size(), Vec3A::new(1f32, 1f32, 2f32));
    /// ```
    pub fn size(&self) -> Vec3A {
        (self.max - self.min).abs()
    }

    /// Applies the given transformation to this [BoundingBox].
    ///
    /// Arguments:
    ///
    /// * `transformation`: The transformation that will be applied.
    ///
    /// # Example
    ///
    /// ```rust
    /// use glam::{Mat4, Vec3, Vec3A};
    /// use meshtext::BoundingBox;
    ///
    /// let mut bbox = BoundingBox::new(
    ///     Vec3A::new(0f32, 0f32, 0f32),
    ///     Vec3A::new(1f32, 1f32, 1f32),
    /// );
    /// let transformed_bbox = BoundingBox::new(
    ///     Vec3A::new(1f32, 0f32, 0f32),
    ///     Vec3A::new(2f32, 1f32, 0.1),
    /// );
    ///
    /// let transform = Mat4::from_scale(Vec3::new(1f32, 1f32, 0.1)) *
    ///     Mat4::from_translation(Vec3::new(1f32, 0f32, 0f32));
    ///
    /// // Apply the transformation.
    /// bbox.transform(&transform);
    ///
    /// assert_eq!(bbox, transformed_bbox);
    /// ```
    pub fn transform(&mut self, transformation: &Mat4) {
        self.min = transformation.transform_point3a(self.min);
        self.max = transformation.transform_point3a(self.max);
    }

    /// Applies the given two-dimensional transformation to this [BoundingBox].
    ///
    /// Arguments:
    ///
    /// * `transformation`: The transformation that will be applied.
    pub(crate) fn transform_2d(&mut self, transformation: &glam::Mat3) {
        let mut min = glam::Vec2::new(self.min.x, self.min.y);
        let mut max = glam::Vec2::new(self.max.x, self.max.y);

        min = transformation.transform_point2(min);
        max = transformation.transform_point2(max);

        self.min = Vec3A::new(min.x, min.y, 0f32);
        self.max = Vec3A::new(max.x, max.y, 0f32);
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            max: Vec3A::new(-0.5f32, -0.5f32, -0.5f32),
            min: Vec3A::new(0.5f32, 0.5f32, 0.5f32),
        }
    }
}
