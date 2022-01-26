//! Main entry point for LensrsCli

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use ocular_cli::application::APP;

/// Boot LensrsCli
fn main() {
    abscissa_core::boot(&APP);
}
