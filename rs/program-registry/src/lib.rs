use std::{collections::BTreeMap, fs, path::Path, sync::OnceLock};

pub use inventory as __inventory;
use inventory::collect;
use sails_idl_gen::{Error as IdlGenError, program};
use sails_idl_meta::ProgramMeta;
use sails_interface_id::{self, runtime::BuildError as CanonicalBuildError};
use thiserror::Error;

#[macro_export]
macro_rules! submit_program_registration {
    ($registration:expr) => {
        $crate::__inventory::submit! {
            $registration
        }
    };
}

/// Result alias used by artifact generation callbacks.
pub type ArtifactResult = core::result::Result<(), ArtifactError>;

/// Registration describing a single Sails program crate.
pub struct ProgramRegistration {
    /// Cargo package name of the crate that produced this registration.
    pub package: &'static str,
    /// Fully qualified Rust type path of the program struct.
    pub type_path: &'static str,
    /// Version of the meta path used in the program implementation.
    pub meta_path_version: MetaPathVersion,
    /// Function pointer writing the program IDL to the supplied path.
    pub write_idl: fn(&Path) -> ArtifactResult,
    /// Function pointer writing the program canonical document to the supplied path.
    pub write_canonical: fn(&Path) -> ArtifactResult,
}

collect!(ProgramRegistration);

static REGISTRY: OnceLock<BTreeMap<&'static str, &'static ProgramRegistration>> = OnceLock::new();

/// Retrieve all registered programs as a lazily initialised map keyed by package.
fn registry() -> &'static BTreeMap<&'static str, &'static ProgramRegistration> {
    REGISTRY.get_or_init(|| {
        let mut map = BTreeMap::new();
        for registration in inventory::iter::<ProgramRegistration> {
            if let Some(existing) = map.insert(registration.package, registration) {
                panic!(
                    "duplicate Sails program registration for package `{}` (existing type `{}`)",
                    registration.package, existing.type_path
                );
            }
        }
        map
    })
}

/// Iterate over every registered program.
pub fn iter() -> impl Iterator<Item = &'static ProgramRegistration> {
    registry().values().copied()
}

/// Lookup a registered program by Cargo package name.
pub fn lookup_by_package(package: &str) -> Result<&'static ProgramRegistration, RegistryError> {
    registry()
        .get(package)
        .copied()
        .ok_or_else(|| RegistryError::NotFound {
            package: package.to_owned(),
        })
}

/// Version of the path used to reach [`sails_idl_meta::ProgramMeta`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaPathVersion {
    V1,
    V2,
}

impl MetaPathVersion {
    /// Returns the path segments expected for this meta version.
    pub fn expected_path(&self) -> &'static [&'static str] {
        match self {
            MetaPathVersion::V1 => &["sails_idl_meta", "ProgramMeta"],
            MetaPathVersion::V2 => &["sails_rs", "meta", "ProgramMeta"],
        }
    }
}

/// Errors surfaced by registry helpers.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("package `{package}` is not registered as a Sails program")]
    NotFound { package: String },
    #[error("failed to write {artifact} for `{package}`: {source}")]
    ArtifactFailed {
        package: String,
        artifact: ArtifactKind,
        #[source]
        source: ArtifactError,
    },
}

/// Errors returned by artifact generation callbacks.
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("{0}")]
    Custom(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ArtifactError {
    /// Construct an arbitrary error message.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

/// Identifier for the type of artifact we attempt to produce.
#[derive(Debug, Clone, Copy)]
pub enum ArtifactKind {
    Idl,
    Canonical,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtifactKind::Idl => "IDL",
            ArtifactKind::Canonical => "canonical document",
        }
    }
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ProgramRegistration {
    /// Convenience wrapper that invokes the IDL writer and maps any error.
    pub fn write_idl(&'static self, path: impl AsRef<Path>) -> Result<(), RegistryError> {
        (self.write_idl)(path.as_ref()).map_err(|source| RegistryError::ArtifactFailed {
            package: self.package.to_owned(),
            artifact: ArtifactKind::Idl,
            source,
        })
    }

    /// Convenience wrapper that invokes the canonical writer and maps any error.
    pub fn write_canonical(&'static self, path: impl AsRef<Path>) -> Result<(), RegistryError> {
        (self.write_canonical)(path.as_ref()).map_err(|source| RegistryError::ArtifactFailed {
            package: self.package.to_owned(),
            artifact: ArtifactKind::Canonical,
            source,
        })
    }
}

pub fn write_idl<P: ProgramMeta + 'static>(path: &Path) -> ArtifactResult {
    program::generate_idl_to_file::<P>(path).map_err(from_idl_error)
}

pub fn write_canonical<P: ProgramMeta + 'static>(path: &Path) -> ArtifactResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = sails_interface_id::ensure_canonical_bytes::<P>().map_err(from_canonical_error)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn from_idl_error(err: IdlGenError) -> ArtifactError {
    ArtifactError::custom(err.to_string())
}

fn from_canonical_error(err: CanonicalBuildError) -> ArtifactError {
    ArtifactError::custom(err.to_string())
}
