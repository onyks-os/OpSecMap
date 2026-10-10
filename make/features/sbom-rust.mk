# sbom-rust.mk — Rust binaries are statically linked: every crate in the
# normal-dependency closure is inside the artifact, so the SBOM lists them
# (module: sbom, rust profile).
SBOM_ARGS += --cargo
