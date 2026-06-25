use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Text,
        }
    }
}

pub fn format_output<T: Serialize + std::fmt::Debug>(data: &T, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(data).unwrap_or_default(),
        OutputFormat::Text => format!("{:#?}", data),
    }
}

pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let mut output = String::new();

    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            output.push_str("  ");
        }
        output.push_str(&format!("{:width$}", header, width = col_widths[i]));
    }
    output.push('\n');

    for &width in &col_widths {
        output.push_str(&"-".repeat(width));
        output.push_str("  ");
    }
    output.push('\n');

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&format!("{:width$}", cell, width = col_widths[i]));
        }
        output.push('\n');
    }

    output
}
