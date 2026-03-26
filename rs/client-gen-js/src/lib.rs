use anyhow::{Context, Result, bail};
use root_generator::RootGenerator;
use sails_idl_parser_v2::{parse_idl, preprocess};
use std::{fs, path::Path};
use type_generator::TypeGenerator;

mod helpers;
mod naming;
mod root_generator;
mod service_generator;
mod type_generator;

#[derive(Default)]
pub enum OutputLayout {
    #[default]
    SingleFile,
    Split {
        types_file: String,
        client_file: String,
    },
}

pub struct IdlPath<'a>(&'a Path);
pub struct IdlString<'a>(&'a str);

pub struct JsClientGenerator<S> {
    idl: S,
    output_layout: OutputLayout,
}

impl<S> JsClientGenerator<S> {
    pub fn with_output_layout(self, output_layout: OutputLayout) -> Self {
        Self {
            output_layout,
            ..self
        }
    }
}

impl<'a> JsClientGenerator<IdlPath<'a>> {
    pub fn from_idl_path(idl_path: &'a Path) -> Self {
        Self {
            idl: IdlPath(idl_path),
            output_layout: OutputLayout::SingleFile,
        }
    }

    fn with_idl(self, idl: &'a str) -> JsClientGenerator<IdlString<'a>> {
        JsClientGenerator {
            idl: IdlString(idl),
            output_layout: self.output_layout,
        }
    }

    pub fn generate(self) -> Result<String> {
        let idl_path = self.idl.0;
        let idl = preprocess::fs::preprocess_from_path(idl_path)
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| format!("Failed to preprocess IDL from {}", idl_path.display()))?;
        self.with_idl(&idl).generate()
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self
            .generate()
            .context("failed to generate TypeScript client")?;
        fs::write(out_path, code).with_context(|| {
            format!("Failed to write generated client to {}", out_path.display())
        })?;
        Ok(())
    }
}

impl<'a> JsClientGenerator<IdlString<'a>> {
    pub fn from_idl(idl: &'a str) -> Self {
        Self {
            idl: IdlString(idl),
            output_layout: OutputLayout::SingleFile,
        }
    }

    pub fn generate(self) -> Result<String> {
        let doc = parse_idl(self.idl.0).context("Failed to parse IDL")?;
        let type_gen = TypeGenerator::new();
        let mut generator = RootGenerator::new(&type_gen);
        let output = generator.generate(&doc);

        match self.output_layout {
            OutputLayout::SingleFile => Ok(output),
            OutputLayout::Split { .. } => {
                bail!("Split output layout is not implemented yet in Phase 1")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sails_idl_parser_v2::preprocess::fs::preprocess_from_path;
    use std::path::Path;

    #[test]
    fn test_js_generator_includes() {
        let path = Path::new("tests/idls/includes/main.idl");
        let result = preprocess_from_path(path).expect("Failed to preprocess IDL");

        assert!(result.contains("struct ResultData"));
        assert!(result.contains("enum Error"));
        assert!(result.contains("service ServiceA"));
        assert!(result.contains("program Main"));
        assert!(result.contains("alias MyResult = ResultData;"));

        let count = result.matches("struct ResultData").count();
        assert_eq!(count, 1, "ResultData should be included only once");
    }
}
