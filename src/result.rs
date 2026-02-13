use std::fmt;

pub enum FormatResult {
    Plain(String),
    Table(Vec<Vec<String>>),
}

impl fmt::Display for FormatResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatResult::Plain(s) => write!(f, "{s}"),
            FormatResult::Table(rows) => {
                if rows.is_empty() {
                    return Ok(());
                }
                let col_count = rows[0].len();
                let mut widths = vec![0usize; col_count];
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < col_count {
                            widths[i] = widths[i].max(cell.len());
                        }
                    }
                }
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i > 0 {
                            write!(f, "  ")?;
                        }
                        write!(f, "{:width$}", cell, width = widths.get(i).copied().unwrap_or(0))?;
                    }
                    writeln!(f)?;
                }
                Ok(())
            }
}
    }
}
