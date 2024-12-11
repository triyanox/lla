use super::FileFilter;
use crate::error::{LlaError, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum FilterOperation {
    And,
    Or,
    Not,
    Xor,
}

pub struct CompositeFilter {
    filters: Vec<Box<dyn FileFilter>>,
    operation: FilterOperation,
}

impl CompositeFilter {
    pub fn new(operation: FilterOperation) -> Self {
        CompositeFilter {
            filters: Vec::new(),
            operation,
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn FileFilter>) {
        self.filters.push(filter);
    }
}

impl FileFilter for CompositeFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        if self.filters.is_empty() {
            return Ok(files.to_vec());
        }

        match self.operation {
            FilterOperation::And => {
                let mut result = files.to_vec();
                for filter in &self.filters {
                    result = filter
                        .filter_files(&result)
                        .map_err(|e| LlaError::Filter(format!("AND operation failed: {}", e)))?;
                }
                Ok(result)
            }
            FilterOperation::Or => {
                let mut result = Vec::new();
                for filter in &self.filters {
                    let filtered = filter
                        .filter_files(files)
                        .map_err(|e| LlaError::Filter(format!("OR operation failed: {}", e)))?;
                    for file in filtered {
                        if !result.contains(&file) {
                            result.push(file);
                        }
                    }
                }
                Ok(result)
            }
            FilterOperation::Not => {
                if self.filters.len() != 1 {
                    return Err(LlaError::Filter(
                        "NOT operation requires exactly one filter".to_string(),
                    ));
                }
                let filtered = self.filters[0]
                    .filter_files(files)
                    .map_err(|e| LlaError::Filter(format!("NOT operation failed: {}", e)))?;
                Ok(files
                    .iter()
                    .filter(|file| !filtered.contains(file))
                    .cloned()
                    .collect())
            }
            FilterOperation::Xor => {
                if self.filters.len() != 2 {
                    return Err(LlaError::Filter(
                        "XOR operation requires exactly two filters".to_string(),
                    ));
                }
                let first = self.filters[0].filter_files(files).map_err(|e| {
                    LlaError::Filter(format!("XOR operation failed on first filter: {}", e))
                })?;
                let second = self.filters[1].filter_files(files).map_err(|e| {
                    LlaError::Filter(format!("XOR operation failed on second filter: {}", e))
                })?;

                Ok(files
                    .iter()
                    .filter(|file| {
                        let in_first = first.contains(file);
                        let in_second = second.contains(file);
                        in_first ^ in_second
                    })
                    .cloned()
                    .collect())
            }
        }
    }
}
