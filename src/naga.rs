use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags},
};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::wgsl_error::WgslError;

const INCLUDE_INSTRUCTION: &'static str = "//!include";
const IGNORE_INSTRUCTION: &'static str = "//!ignore";

pub struct Naga {
    validator: naga::valid::Validator,
}

impl Naga {
    pub fn new() -> Self {
        Self {
            validator: naga::valid::Validator::new(ValidationFlags::all(), Capabilities::all()),
        }
    }

    pub fn validate_wgsl(&mut self, path: &Path) -> Result<(), WgslError> {
        let mut shader = std::fs::read_to_string(&path).map_err(WgslError::from)?;
        
        // Process //!include statements
        for line in shader.clone().lines() {
            if line.starts_with(IGNORE_INSTRUCTION) {
                continue;
            } else if line.starts_with(INCLUDE_INSTRUCTION) {
                for include in line.split_whitespace().skip(1) {
                    let includepath = std::path::PathBuf::from(path.parent().unwrap().join(include));
                    let contents = std::fs::read_to_string(includepath.clone()).map_err(WgslError::from)?;
                    shader.insert_str(0, &contents);
                };
            };
        };
        
        let module =
            wgsl::parse_str(&shader).map_err(|err| WgslError::from_parse_err(err, &shader))?;

        if let Err(error) = self.validator.validate(&module) {
            Err(WgslError::ValidationErr { src: shader, error })
        } else {
            Ok(())
        }
    }

    pub fn get_wgsl_tree(&mut self, path: &Path) -> Result<WgslTree, WgslError> {
        let shader = std::fs::read_to_string(&path).map_err(WgslError::from)?;
        let module =
            wgsl::parse_str(&shader).map_err(|err| WgslError::from_parse_err(err, &shader))?;

        let mut types = Vec::new();
        let mut global_variables = Vec::new();
        let mut functions = Vec::new();

        for (_, ty) in module.types.iter() {
            if let Some(name) = &ty.name {
                types.push(name.clone())
            }
        }

        for (_, var) in module.global_variables.iter() {
            if let Some(name) = &var.name {
                global_variables.push(name.clone())
            }
        }

        for (_, f) in module.functions.iter() {
            if let Some(name) = &f.name {
                functions.push(name.clone())
            }
        }

        Ok(WgslTree {
            types,
            global_variables,
            functions,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WgslTree {
    types: Vec<String>,
    global_variables: Vec<String>,
    functions: Vec<String>,
}
