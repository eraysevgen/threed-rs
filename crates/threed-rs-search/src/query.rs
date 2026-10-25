/// Neighborhood search method definition matching original configuration (KNN or Radius).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeighborhoodMethod {
    /// K-Nearest Neighbors search with fixed count `k`.
    Knn { k: usize },

    /// Radius / Range search with distance `radius` and optional maximum return limit `max_k`.
    Radius { radius: f64, max_k: Option<usize> },
}

impl NeighborhoodMethod {
    /// Creates a KNN search specification.
    #[inline]
    pub fn knn(k: usize) -> Self {
        Self::Knn { k }
    }

    /// Creates a Radius search specification with an optional maximum neighbor cutoff.
    #[inline]
    pub fn radius(radius: f64, max_k: Option<usize>) -> Self {
        Self::Radius { radius, max_k }
    }

    /// Validates the search parameters.
    pub fn validate(&self) -> Result<(), crate::error::SearchError> {
        match *self {
            Self::Knn { k } => {
                if k == 0 {
                    return Err(crate::error::SearchError::ZeroNeighborsRequested);
                }
            }
            Self::Radius { radius, max_k } => {
                if radius <= 0.0 || radius.is_nan() {
                    return Err(crate::error::SearchError::InvalidRadius(radius));
                }
                if let Some(m) = max_k {
                    if m == 0 {
                        return Err(crate::error::SearchError::InvalidParameter(
                            "max_k in radius search must be greater than zero".into(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// A neighbor match item containing the host point index and squared Euclidean distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeighborItem {
    /// Index of the neighbor point in the host point cloud.
    pub index: usize,
    /// Squared Euclidean distance to the query point.
    pub distance_squared: f64,
}

impl NeighborItem {
    /// Creates a new `NeighborItem`.
    #[inline]
    pub fn new(index: usize, distance_squared: f64) -> Self {
        Self {
            index,
            distance_squared,
        }
    }

    /// Returns the true Euclidean distance: `sqrt(distance_squared)`.
    #[inline]
    pub fn distance(&self) -> f64 {
        self.distance_squared.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_validation() {
        assert!(NeighborhoodMethod::knn(10).validate().is_ok());
        assert!(NeighborhoodMethod::knn(0).validate().is_err());

        assert!(NeighborhoodMethod::radius(0.5, Some(100))
            .validate()
            .is_ok());
        assert!(NeighborhoodMethod::radius(0.0, None).validate().is_err());
        assert!(NeighborhoodMethod::radius(-0.2, None).validate().is_err());
        assert!(NeighborhoodMethod::radius(0.5, Some(0)).validate().is_err());
    }

    #[test]
    fn test_neighbor_item() {
        let item = NeighborItem::new(42, 25.0);
        assert_eq!(item.index, 42);
        assert_eq!(item.distance_squared, 25.0);
        assert_eq!(item.distance(), 5.0);
    }
}
