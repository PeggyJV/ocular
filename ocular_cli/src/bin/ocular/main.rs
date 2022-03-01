//! Main entry point for OcularCli

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use ocular_cli::application::APP;

/// Boot OcularCli
fn main() {
    abscissa_core::boot(&APP);
}
