# ThreeD-Rust (threed-rs)

[![Rust CI](https://github.com/eraysevgen/threed-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/eraysevgen/threed-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platforms](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-blue)](https://github.com/eraysevgen/threed-rs/releases)

A high-performance, modern Rust-native implementation of [threed](https://github.com/eraysevgen/threed) for 3D point cloud feature extraction.

Designed from the ground up for massive point clouds, `threed-rs` delivers memory-safe, multi-threaded local geometric, height, and density feature extraction without heavy C++ dependencies like PCL, VTK, or CMake.

---

## ⚡ Features & Advantages

- **Zero Heavy C++ Dependencies:** Pure Rust ecosystem replacing PCL, nanoflann, cnpy, and OpenMP with `kiddo`, `nalgebra`, `rayon`, and `ndarray`.
- **Cross-Platform Out of the Box:** Native binary builds for Linux (x86_64), Windows (x86_64), and macOS (both Apple Silicon M-series ARM64 and Intel x86_64).
- **Ultra-Fast Spatial Indexing:** Powered by [`kiddo`](https://crates.io/crates/kiddo) v6 for memory-efficient k-d tree KNN and spherical radius neighborhood queries.
- **Multithreaded Parallel Execution:** Data-parallel feature computation and batch queries accelerated with [`rayon`](https://crates.io/crates/rayon).
- **Comprehensive File Format Support:** Direct reading and writing of ASPRS LAS/LAZ (v1.4 with projection offsets), Stanford PLY (ASCII & Binary Little Endian), and NumPy `.npy` export via `ndarray-npy`.
- **Drop-in Configuration Compatibility:** Fully compatible with original C++ `config.yaml` workflows while offering an expressive command-line interface.

---

## 📊 Extracted Features

The pipeline extracts up to 18 point cloud features (9 geometric + 5 height + 1 density + 3 spatial coordinates):

### 1. Geometric Features (9 3D Covariance / PCA Eigenvalues: $\lambda_1 \ge \lambda_2 \ge \lambda_3$)
| Feature | Formula / Definition | Description |
| :--- | :--- | :--- |
| **Linearity** | $(\lambda_1 - \lambda_2) / \lambda_1$ | Measures wire/line-like linear structures |
| **Planarity** | $(\lambda_2 - \lambda_3) / \lambda_1$ | Measures planar surfaces (facades, ground, roofs) |
| **Scattering** | $\lambda_3 / \lambda_1$ | Sphericity / 3D volumetric dispersion |
| **Omnivariance** | $(\lambda_1 \cdot \lambda_2 \cdot \lambda_3)^{1/3}$ | Geometric mean of spatial variance |
| **Anisotropy** | $(\lambda_1 - \lambda_3) / \lambda_1$ | Directional bias in point distribution |
| **Eigenentropy** | $-\sum_{i=1}^3 e_i \ln(e_i)$ | Disorder / Shannon entropy of normalized eigenvalues |
| **Sum of Eigenvalues** | $\lambda_1 + \lambda_2 + \lambda_3$ | Total neighborhood variance |
| **Surface Variation** | $\lambda_3 / (\sum \lambda_i)$ | Change in surface orientation |
| **Curvature** | $\lambda_3 / (\lambda_1 + \lambda_2 + \lambda_3)$ | Local surface curvature proxy |

### 2. Height Features (5 Height Statistics)
| Feature | Description |
| :--- | :--- |
| **Z-Range** | Maximum Z $-$ Minimum Z in local neighborhood |
| **Z-Mean** | Difference between query point Z and neighborhood mean Z |
| **Z-Std** | Standard deviation of neighborhood Z values |
| **Z-Variance** | Variance of neighborhood Z values |
| **Z-Skewness** | Third standardized moment (asymmetry) of neighborhood Z distribution |

### 3. Density & Spatial (Optional)
- **Density (1):** Number of neighbor points found in radius search or local query radius.
- **Coordinates (3):** Appends query point coordinates $(X, Y, Z)$ for seamless downstream visualization.

---

## 🏗️ Workspace Architecture

```
threed-rs
├── Cargo.toml
├── config.example.yaml
├── .github/workflows/
│   ├── ci.yml               # Automated CI for Linux, Windows, and macOS (Apple Silicon)
│   └── release.yml          # Multi-platform release binary packaging
├── crates/
│   ├── threed-rs-core       # Fundamental data models (Point3D, BoundingBox, PointCloud, FeatureConfig)
│   ├── threed-rs-search     # Kiddo-based spatial k-d tree indexing (KNN, Radius, Batch search)
│   ├── threed-rs-features   # Covariance matrix, PCA decomposition, 9 geometric + 5 height features
│   ├── threed-rs-io         # LAS / LAZ reader-writer, PLY parser-writer, NumPy .npy export
│   └── threed-rs-cli        # CLI application, YAML configuration parser, and workflow orchestrator
└── tests/                   # End-to-end integration tests
```

---

## 🚀 Quick Start

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2021, Rust $\ge$ 1.80)

### Installation

From source:
```bash
git clone https://github.com/eraysevgen/threed-rs.git
cd threed-rs
cargo build --release -p threed-rs-cli
```
The compiled executable will be located at `target/release/threed-rs` (or `threed-rs.exe` on Windows).

Precompiled binary packages are also available directly on the [GitHub Releases](https://github.com/eraysevgen/threed-rs/releases) page for Linux, Windows, and macOS (Apple Silicon & Intel).

---

## 💻 CLI Usage

### 1. Generate a Template Configuration File
```bash
threed-rs generate-config --out config.yaml
```

### 2. Run with YAML Configuration
```bash
threed-rs --config config.yaml
```

### 3. Run Directly with Command Line Arguments
```bash
# Radius search with density and height features
threed-rs \
  --host input.las \
  --out features.npy \
  --method radius \
  --radius 0.25 \
  --max-k 512 \
  --add-height true \
  --add-density true

# K-Nearest Neighbors (KNN) search on a PLY file
threed-rs \
  --host scan.ply \
  --out scan_features.npy \
  --method knn \
  -k 16 \
  --threads 8
```

### Command Line Options
```text
Options:
  -c, --config <CONFIG>            Path to YAML configuration file
      --host <HOST>                Host point cloud file (.las, .laz, .ply)
      --query <QUERY>              Query point cloud file (.las, .laz, .ply) - defaults to host
  -o, --out <OUT>                  Output NumPy (.npy) file
  -m, --method <METHOD>            Neighborhood search method ('radius' or 'knn')
  -r, --radius <RADIUS>            Search radius (for radius search)
      --max-k <MAX_K>              Maximum number of neighbors (for radius search)
  -k <K>                           Number of nearest neighbors (for knn search)
      --add-height <ADD_HEIGHT>    Add 5 height features [possible values: true, false]
      --add-density <ADD_DENSITY>  Include 1 density feature [possible values: true, false]
      --add-xyz <ADD_XYZ>          Append point XYZ coordinates to features [possible values: true, false]
  -t, --threads <THREADS>          Number of worker threads (0 = auto-detect all cores)
  -h, --help                       Print help
  -V, --version                    Print version
```

---

## ⚙️ Configuration Schema (`config.yaml`)

```yaml
files:
  host_file   : "data/host_data.las"      # Base point cloud for spatial indexing
  query_file  : "data/host_data.las"      # Query point cloud (omitted or null = same as host)
  out_file    : "data/output.npy"         # Output 2D NumPy feature matrix

neighborhood:
  name        : "radius"                  # Search method: 'radius' or 'knn'
  engine      : "cpu"                     # Spatial engine: 'cpu'
  radius      : 0.25                      # Search radius (if method is radius)
  max_k       : 512                       # Maximum neighbor limit
  k           : 8                         # Nearest neighbors count (if method is knn)
  batch_size  : 65536                     # Processing batch size

features:
  add_height  : false                     # Compute 5 height features (default: false)
  add_density : true                      # Compute 1 density feature (default: true)
  add_xyz     : false                     # Append (X, Y, Z) coordinates (default: false)

params:
  num_threads : 0                         # Worker threads (0 = all logical cores)
```

---

## 🧪 Testing & Verification

Run the test suite across all 5 workspace crates:
```bash
cargo test --all
```

Check code quality with Clippy:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Verify formatting:
```bash
cargo fmt --all -- --check
```

---

## 🗺️ Roadmap
- [x] `threed-rs-core`: Fundamental point cloud data structures and bounding boxes
- [x] `threed-rs-search`: Kiddo v6 spatial KD-Tree indexing and parallel batch querying
- [x] `threed-rs-features`: Covariance matrix, PCA decomposition, 9 geometric + 5 height + density features
- [x] `threed-rs-io`: LAS/LAZ, PLY, and NumPy `.npy` export
- [x] `threed-rs-cli`: Interactive CLI, YAML config parsing, progress tracking, and orchestration
- [x] Multi-platform CI & release workflows (Ubuntu Linux, Windows, macOS Apple Silicon / Intel)

---

## 📚 Publications & Citation

If you use `threed-rs` or the multi-scale point cloud classification methodology in your research, please cite our papers:

```bibtex
@Article{rs15153787,
  AUTHOR         = {Sevgen, Eray and Abdikan, Saygin},
  TITLE          = {Classification of Large-Scale Mobile Laser Scanning Data in Urban Area with LightGBM},
  JOURNAL        = {Remote Sensing},
  VOLUME         = {15},
  YEAR           = {2023},
  NUMBER         = {15},
  ARTICLE-NUMBER = {3787},
  URL            = {https://www.mdpi.com/2072-4292/15/15/3787},
  ISSN           = {2072-4292},
  DOI            = {10.3390/rs15153787}
}

@Article{isprs-archives-XLVIII-M-1-2023-587-2023,
  AUTHOR         = {Sevgen, E. and Abdikan, S.},
  TITLE          = {POINT-WISE CLASSIFICATION OF HIGH-DENSITY UAV-LIDAR DATA USING GRADIENT BOOSTING MACHINES},
  JOURNAL        = {The International Archives of the Photogrammetry, Remote Sensing and Spatial Information Sciences},
  VOLUME         = {XLVIII-M-1-2023},
  PAGES          = {587--593},
  YEAR           = {2023},
  URL            = {https://isprs-archives.copernicus.org/articles/XLVIII-M-1-2023/587/2023/},
  DOI            = {10.5194/isprs-archives-XLVIII-M-1-2023-587-2023}
}
```

---

## 🤖 Acknowledgments

This project was architected, engineered, and migrated from C++ to Rust with the assistance of **Google DeepMind Antigravity**.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
