use std::fmt::Debug;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::ValueHint;
use tokio::sync::mpsc;

use mesh::Mesh;

use crate::sdf::SDFSurface;
use crate::sdf::wasm::load;
use crate::sdf::wasm::load::spawn_async;

mod mesh;

#[cfg(feature = "isosurface")]
mod isosurface;

/// Export your SDF by converting it to a triangle mesh compatible with most 3D modelling tools.
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliMesher {
    /// Input file or URL: .wasm file representing a SDF.
    /// If using the GUI, this will be overwritten with the current root SDF.
    #[clap(short, long = "input", default_value = "")]
    pub input: String,
    /// Output file: .ply 3D model made of triangles. Set to "-" to write to stdout/GUI window.
    /// WARNING: Output to GUI window may be too laggy for large models.
    #[clap(short, long = "output", parse(from_os_str), value_hint = ValueHint::FilePath, default_value = "mesh.ply")]
    pub output_file: PathBuf,
    #[clap(flatten)]
    pub cfg: Config,
    #[clap(subcommand)]
    pub mesher: Meshers,
}

impl CliMesher {
    /// Runs the CLI for the mesher, using all the configured parameters.
    pub async fn run_cli(self) -> anyhow::Result<()> {
        let output_is_stdout = self.output_file.to_str()
            .map(|s| s.is_empty() || s.eq("-")).unwrap_or(false);
        if output_is_stdout { // This if-else can't be merged because of async/await?
            // Buffer writes for faster performance
            let f = io::stdout();
            let mut f = BufWriter::new(f);
            // Run as usual
            self.run_custom_out(&mut f).await?;
        } else {
            // Check that the output file does not exist yet or fail
            let output_file = self.output_file.to_str().unwrap().to_string();
            if File::open(&output_file).is_ok() {
                anyhow::bail!("Output file already exists");
            }
            // Create/truncate the output file or fail
            let f = File::create(&output_file)?;
            // Buffer writes for faster performance
            let mut f = BufWriter::new(f);
            // Run as usual
            self.run_custom_out(&mut f).await?;
        };
        Ok(())
    }

    /// Runs the mesher and writes the output to the given writer instead of the configured file.
    pub async fn run_custom_out<W: Write>(self, w: &mut W) -> anyhow::Result<usize> {
        // Start loading input SDF (using common code with the app)
        tracing::info!("Loading SDF from {:?}...", self.input);
        let (sender_of_updates, mut receiver_of_updates) = mpsc::channel(1);
        spawn_async(async move { load::load_sdf_from_path_or_url(sender_of_updates, self.input.clone()) }, false);
        // Wait for the loaded SDF to be ready
        let input_sdf = receiver_of_updates
            .recv().await.ok_or_else(|| anyhow::anyhow!("No SDF found"))?
            .recv().await.ok_or_else(|| anyhow::anyhow!("No SDF found"))?;
        drop(receiver_of_updates);
        // Apply the meshing algorithm as configured
        tracing::info!("Running the meshing algorithm with {:?} {:?}...", self.cfg, self.mesher);
        // TODO: Progress reporting + ETA?
        // TODO: Async for avoiding freezes on wasm32
        let mut mesh = self.mesher.mesh(&input_sdf, self.cfg);
        // Post-process the mesh to get the materials
        tracing::info!("Post-processing the mesh ({} vertices, {} triangles)...", mesh.vertices.len(), mesh.indices.len() / 3);
        mesh.postproc(&input_sdf);
        // Write the mesh to the output file or fail
        tracing::info!("Serializing output mesh...");
        Ok(mesh.serialize_ply(w)?)
    }
}

/// Common config shared by all meshers
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The maximum number of voxels or cells used for the largest axis of the volume.
    /// Some algorithms require this to be a power of two.
    #[clap(short = 'v', long, default_value = "64")]
    pub max_voxels_per_axis: usize,
}

impl Default for Config {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from([""])
    }
}

pub trait Mesher: Debug {
    /// Mesh reconstructs a mesh from a [`sdf_viewer::sdf::SDFSurface`] trait.
    fn mesh(&self, sdf: &dyn SDFSurface, cfg: Config) -> Mesh;
}

/// Meshers holds the list of currently implemented meshing algorithms.
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Meshers {
    #[cfg(feature = "isosurface")]
    MarchingCubes,
    #[cfg(feature = "isosurface")]
    LinearHashedMarchingCubes,
    // #[cfg(feature = "isosurface")]
    // ExtendedMarchingCubes,
    #[cfg(feature = "isosurface")]
    DualContouringMinimizeQEF,
    #[cfg(feature = "isosurface")]
    DualContouringParticleBasedMinimization,
}

impl Default for Meshers {
    fn default() -> Self {
        Self::MarchingCubes
    }
}

impl Mesher for Meshers {
    fn mesh(&self, s: &dyn SDFSurface, cfg: Config) -> Mesh {
        match self {
            #[cfg(feature = "isosurface")]
            Meshers::MarchingCubes => isosurface::mesh(0, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::LinearHashedMarchingCubes => isosurface::mesh(1, cfg, s),
            // #[cfg(feature = "isosurface")]
            // Meshers::ExtendedMarchingCubes => isosurface::mesh(2, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::DualContouringMinimizeQEF => isosurface::mesh(3, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::DualContouringParticleBasedMinimization => isosurface::mesh(4, cfg, s),
        }
    }
}

