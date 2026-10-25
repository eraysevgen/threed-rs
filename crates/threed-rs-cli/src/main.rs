mod config;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;
use threed_rs_features::compute_features;
use threed_rs_io::{read_point_cloud, write_features_npy, write_metadata_yaml};
use threed_rs_search::KdTreeIndex;

use crate::config::AppConfig;

#[derive(Parser, Debug)]
#[command(
    name = "threed-rs",
    author = "Eray Sevgen <eraysevgen@gmail.com>",
    version = "0.1.0",
    about = "High-performance 3D Point Cloud Local Feature Extraction in Rust"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to YAML configuration file
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,

    /// Host point cloud file (.las, .laz, .ply)
    #[arg(long = "host")]
    host: Option<PathBuf>,

    /// Query point cloud file (.las, .laz, .ply) - defaults to host file
    #[arg(long = "query")]
    query: Option<PathBuf>,

    /// Output NumPy (.npy) file
    #[arg(short = 'o', long = "out")]
    out: Option<PathBuf>,

    /// Neighborhood search method ('radius' or 'knn')
    #[arg(short = 'm', long = "method")]
    method: Option<String>,

    /// Search radius (for radius search)
    #[arg(short = 'r', long = "radius")]
    radius: Option<f64>,

    /// Maximum number of neighbors (for radius search)
    #[arg(long = "max-k")]
    max_k: Option<usize>,

    /// Number of nearest neighbors (for knn search)
    #[arg(short = 'k')]
    k: Option<usize>,

    /// Add 5 height features (z_range, z_mean, z_std, z_variance, z_skewness)
    #[arg(long = "add-height")]
    add_height: Option<bool>,

    /// Include 1 density feature
    #[arg(long = "add-density")]
    add_density: Option<bool>,

    /// Append point XYZ coordinates to features
    #[arg(long = "add-xyz")]
    add_xyz: Option<bool>,

    /// Number of worker threads (0 = all available cores)
    #[arg(short = 't', long = "threads")]
    threads: Option<usize>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates a sample configuration YAML file
    GenerateConfig {
        /// Target output file path
        #[arg(short = 'o', long = "out", default_value = "config.yaml")]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check for subcommands
    if let Some(Commands::GenerateConfig { out }) = cli.command {
        let default_config = AppConfig::default();
        let yaml_str = default_config.to_yaml_string()?;
        std::fs::write(&out, yaml_str)
            .with_context(|| format!("Failed to write sample config to {}", out.display()))?;
        println!("Sample configuration generated at: {}", out.display());
        return Ok(());
    }

    // Load base configuration
    let mut config = if let Some(config_path) = cli.config {
        println!("Loading configuration from: {}", config_path.display());
        AppConfig::from_yaml_file(config_path)?
    } else {
        AppConfig::default()
    };

    // Override config with CLI arguments if provided
    if let Some(h) = cli.host {
        config.files.host_file = h;
    }
    if let Some(q) = cli.query {
        config.files.query_file = Some(q);
    }
    if let Some(o) = cli.out {
        config.files.out_file = o;
    }
    if let Some(m) = cli.method {
        config.neighborhood.name = m;
    }
    if let Some(r) = cli.radius {
        config.neighborhood.radius = r;
    }
    if let Some(mk) = cli.max_k {
        config.neighborhood.max_k = mk;
    }
    if let Some(k) = cli.k {
        config.neighborhood.k = k;
    }
    if let Some(ah) = cli.add_height {
        config.features.add_height = ah;
    }
    if let Some(ad) = cli.add_density {
        config.features.add_density = ad;
    }
    if let Some(axyz) = cli.add_xyz {
        config.features.add_xyz = axyz;
    }
    if let Some(t) = cli.threads {
        config.params.num_threads = t;
    }

    config.validate()?;

    // Configure Rayon thread pool if specified
    if config.params.num_threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(config.params.num_threads)
            .build_global()
            .ok(); // Ignore if pool already initialized
        println!(
            "Configured thread pool with {} worker threads",
            config.params.num_threads
        );
    } else {
        println!("Using all available CPU cores for parallel processing");
    }

    let overall_start = Instant::now();

    // 1. Read Host Point Cloud
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    spinner.set_message(format!(
        "Reading host cloud: {}",
        config.files.host_file.display()
    ));
    let start = Instant::now();
    let host_cloud = read_point_cloud(&config.files.host_file).with_context(|| {
        format!(
            "Failed to read host cloud: {}",
            config.files.host_file.display()
        )
    })?;
    spinner.finish_with_message(format!(
        "Host cloud loaded: {} points ({:.2?})",
        host_cloud.len(),
        start.elapsed()
    ));

    // 2. Read Query Point Cloud (or reuse host)
    let query_file = config.resolved_query_file();
    let query_cloud = if query_file == config.files.host_file.as_path() {
        println!("Query cloud is identical to host cloud");
        host_cloud.clone()
    } else {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        spinner.set_message(format!("Reading query cloud: {}", query_file.display()));
        let start = Instant::now();
        let q_cloud = read_point_cloud(query_file)
            .with_context(|| format!("Failed to read query cloud: {}", query_file.display()))?;
        spinner.finish_with_message(format!(
            "Query cloud loaded: {} points ({:.2?})",
            q_cloud.len(),
            start.elapsed()
        ));
        q_cloud
    };

    // 3. Build Spatial Index (KdTree)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    spinner.set_message("Building KdTree spatial index...");
    let start = Instant::now();
    let index = KdTreeIndex::build_from_cloud(&host_cloud)
        .context("Failed to build spatial index from host cloud")?;
    spinner.finish_with_message(format!("KdTree built in {:.2?}", start.elapsed()));

    // 4. Spatial Neighborhood Search
    let method = config.to_neighborhood_method()?;
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.yellow} {msg}")?);
    spinner.set_message(format!("Searching neighborhoods using {:?}...", method));
    let start = Instant::now();
    let neighbor_indices = index
        .batch_search_indices(&query_cloud, &method)
        .context("Failed during neighborhood search")?;
    spinner.finish_with_message(format!(
        "Neighborhood search completed for {} points in {:.2?}",
        neighbor_indices.len(),
        start.elapsed()
    ));

    // 5. Feature Extraction Pipeline
    let feature_cfg = config.to_feature_config();
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.magenta} {msg}")?);
    spinner.set_message(format!(
        "Computing {} features (parallel)...",
        feature_cfg.total_features()
    ));
    let start = Instant::now();
    let features = compute_features(&host_cloud, &query_cloud, &neighbor_indices, feature_cfg)
        .context("Failed during feature extraction")?;
    spinner.finish_with_message(format!(
        "Features computed for {} points in {:.2?}",
        features.len(),
        start.elapsed()
    ));

    // 6. Export to NumPy (.npy)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
    spinner.set_message(format!(
        "Exporting features to: {}",
        config.files.out_file.display()
    ));
    let start = Instant::now();
    write_features_npy(&config.files.out_file, &features).with_context(|| {
        format!(
            "Failed to export features to {}",
            config.files.out_file.display()
        )
    })?;
    spinner.finish_with_message(format!("Features saved in {:.2?}", start.elapsed()));

    // 7. Write companion config YAML
    let yaml_meta_path = config.files.out_file.with_extension("yaml");
    if let Ok(yaml_content) = config.to_yaml_string() {
        let _ = write_metadata_yaml(&yaml_meta_path, &yaml_content);
    }

    println!("\n================== Processing Complete ==================");
    println!("Total Points Processed: {}", query_cloud.len());
    println!("Feature Vector Dimension: {}", feature_cfg.total_features());
    println!("Feature Columns: {:?}", feature_cfg.column_names());
    println!("Output File: {}", config.files.out_file.display());
    println!("Metadata YAML: {}", yaml_meta_path.display());
    println!("Total Wall Time: {:.2?}", overall_start.elapsed());
    println!("=========================================================\n");

    Ok(())
}
