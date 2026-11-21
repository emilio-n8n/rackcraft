//! Errors from stencil parsing, conversion and export.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias for the I/O crate.
pub type Result<T> = std::result::Result<T, IoError>;

/// Errors surfaced by `.vssx` parsing, `.vss` conversion and BOM exports.
#[derive(Debug, Error)]
pub enum IoError {
    /// Filesystem error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not a valid ZIP/OPC package.
    #[error("invalid Visio package: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// Visio XML is malformed.
    #[error("invalid Visio XML in {part}: {message}")]
    Xml {
        /// Package part being parsed.
        part: String,
        /// Human-readable parser diagnostic.
        message: String,
    },
    /// A required OPC part is absent.
    #[error("not a Visio stencil: missing {0}")]
    MissingPart(String),
    /// The package contains no usable master.
    #[error("the stencil contains no masters")]
    NoMasters,
    /// A legacy binary `.vss` file was detected.
    #[error(
        "legacy binary Visio stencil (.vss) detected{detail}. RackCraft reads .vssx natively. \
         Convert it first with `soffice --headless --convert-to vssx --outdir <folder> <file>` \
         (LibreOffice) or re-save it as .vssx from Visio, then import the .vssx file."
    )]
    LegacyVss {
        /// Additional information extracted from the compound file.
        detail: String,
    },
    /// LibreOffice conversion failed.
    #[error("LibreOffice .vss conversion failed: {0}")]
    Conversion(String),
    /// Spreadsheet export failed.
    #[error("Excel export failed: {0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),
    /// CSV export failed.
    #[error("CSV export failed: {0}")]
    Csv(#[from] csv::Error),
    /// Invalid export target.
    #[error("invalid export target: {0}")]
    ExportTarget(PathBuf),
    /// The bytes are not a supported stencil container.
    #[error("unrecognized stencil format: {0}")]
    UnknownFormat(String),
    /// Document JSON passed to the exporter is invalid.
    #[error("invalid document JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl IoError {
    /// Attach the current part name to an XML error.
    pub(crate) fn xml(part: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Xml {
            part: part.into(),
            message: message.into(),
        }
    }
}
