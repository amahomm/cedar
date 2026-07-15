/*
 * Copyright Cedar Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use clap::Args;

use crate::{load_entities, CedarExitCode, EntitiesFormat};

#[derive(Args, Debug)]
pub struct VisualizeArgs {
    /// File containing a Cedar entity hierarchy
    #[arg(long = "entities", value_name = "FILE")]
    pub entities_file: String,
    /// Entities format
    #[cfg(feature = "cedar-entity-syntax")]
    #[arg(long, value_enum, default_value_t)]
    pub entities_format: EntitiesFormat,
}

pub fn visualize(args: &VisualizeArgs) -> CedarExitCode {
    let format = {
        #[cfg(feature = "cedar-entity-syntax")]
        { args.entities_format }
        #[cfg(not(feature = "cedar-entity-syntax"))]
        { EntitiesFormat::default() }
    };
    match load_entities(&args.entities_file, format, None) {
        Ok(entities) => {
            println!("{}", entities.to_dot_str());
            CedarExitCode::Success
        }
        Err(report) => {
            eprintln!("{report:?}");
            CedarExitCode::Failure
        }
    }
}
