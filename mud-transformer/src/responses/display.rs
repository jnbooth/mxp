use std::fmt;

use mxp::escape::ansi::CSI;

/// Formats a DECRPDE response.
#[derive(Copy, Clone, Debug)]
pub struct DisplayedExtentReport {
    /// Number of lines of the current page displayed excluding the status line.
    rows: u16,
    /// Number of columns of the current page displayed.
    columns: u16,
    /// Column number displayed in the left-most column.
    first_column: u16,
    /// Line number displayed in the top line.
    first_row: u16,
    /// Page number displayed.
    page: usize,
}

impl fmt::Display for DisplayedExtentReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            rows,
            columns,
            first_column,
            first_row,
            page,
        } = self;
        write!(
            f,
            "{CSI}{rows};{columns};{first_column};{first_row};{page};\"w"
        )
    }
}
