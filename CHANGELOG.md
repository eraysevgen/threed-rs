# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-26

### Added
- **Core Data Structures (`threed-rs-core`)**:
  - `Point3D` with coordinate indexing, distance metrics, and seamless `nalgebra` vector conversion.
  - `BoundingBox` calculation, point containment checks, and bounding box expansion.
  - `PointCloud` memory container with spatial offsets and coordinate array exports.
  - `FeatureConfig` supporting granular feature selection and column name mapping.

- **Spatial Indexing (`threed-rs-search`)**:
  - KD-Tree implementation powered by `kiddo` v6 (`ImmutableKdTree<f64, 3>`).
  - K-Nearest Neighbors (KNN) search with configurable $K$.
  - Spherical radius search with optional `max_k` capping.
  - Data-parallel batch spatial search accelerated by `rayon`.

- **Feature Extraction (`threed-rs-features`)**:
  - Covariance matrix computation and PCA eigenvalue/eigenvector decomposition via `nalgebra::SymmetricEigen`.
  - 9 Geometric features: Linearity, Planarity, Scattering, Omnivariance, Anisotropy, Eigenentropy, Sum of Eigenvalues, Surface Variation, and Curvature.
  - 5 Height features: Z-Range, Z-Mean, Z-Std, Z-Variance, and Z-Skewness.
  - 1 Density feature: Local neighborhood point count or query radius.
  - Optional $(X, Y, Z)$ spatial coordinate appending for downstream visualization.
  - Multi-threaded `FeaturePipeline` orchestrator using Rayon.

- **File I/O & Export (`threed-rs-io`)**:
  - ASPRS LAS and compressed LAZ (v1.4) reader and writer preserving spatial transform offsets.
  - Stanford PLY reader and writer supporting both ASCII and Binary Little Endian encodings.
  - Direct 2D NumPy (`.npy`) export via `ndarray-npy`.
  - Automatic format dispatching by file extension (`read_point_cloud`, `write_point_cloud`).
  - Companion metadata YAML generation alongside feature exports.

- **Command Line Interface (`threed-rs-cli`)**:
  - Expressive CLI using `clap` v4 with `--config`, direct flag overrides, and `-t/--threads` support.
  - Subcommand `generate-config` for bootstrapping template YAML files.
  - Animated terminal progress spinner via `indicatif`.
  - Comprehensive execution timing and summary statistics reporting.

- **CI & Quality Assurance**:
  - Multi-platform GitHub Actions CI matrix covering both `ubuntu-latest` and `windows-latest`.
  - End-to-end integration test validating synthetic point cloud feature extraction pipeline.
