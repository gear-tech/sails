use anyhow::{Context, Result};
use root_generator::RootGenerator;
use sails_idl_parser_v2::{FsLoader, GitLoader, parse_idl, preprocess, visitor};
use std::{collections::HashMap, fs, io::Write, path::Path};

mod ctor_generators;
mod events_generator;
mod helpers;
mod mock_generator;
mod root_generator;
mod service_generators;
mod type_generators;

const SAILS: &str = "sails_rs";

pub struct IdlPath<'ast>(&'ast Path);
pub struct IdlString<'ast>(&'ast str);
pub struct ClientGenerator<'ast, S> {
    sails_path: Option<&'ast str>,
    mocks_feature_name: Option<&'ast str>,
    external_types: HashMap<&'ast str, &'ast str>,
    no_derive_traits: bool,
    with_no_std: bool,
    client_path: Option<&'ast Path>,
    idl: S,
}

impl<'ast, S> ClientGenerator<'ast, S> {
    pub fn with_mocks(self, mocks_feature_name: &'ast str) -> Self {
        Self {
            mocks_feature_name: Some(mocks_feature_name),
            ..self
        }
    }

    pub fn with_sails_crate(self, sails_path: &'ast str) -> Self {
        Self {
            sails_path: Some(sails_path),
            ..self
        }
    }

    pub fn with_no_std(self, with_no_std: bool) -> Self {
        Self {
            with_no_std,
            ..self
        }
    }

    /// Add an map from IDL type to crate path
    ///
    /// Instead of generating type in client code, use type path from external crate
    ///
    /// # Example
    ///
    /// Following code generates `use my_crate::MyParam as MyFuncParam;`
    /// ```
    /// let code = sails_client_gen_v2::ClientGenerator::from_idl("<idl>")
    ///     .with_external_type("MyFuncParam", "my_crate::MyParam");
    /// ```
    pub fn with_external_type(self, name: &'ast str, path: &'ast str) -> Self {
        let mut external_types = self.external_types;
        external_types.insert(name, path);
        Self {
            external_types,
            ..self
        }
    }

    /// Derive only nessessary [`parity_scale_codec::Encode`], [`parity_scale_codec::Decode`], [`scale_info::TypeInfo`] and [`sails_reflect_hash::ReflectHash`] traits for the generated types
    ///
    /// By default, types additionally derive [`PartialEq`], [`Clone`] and [`Debug`]
    pub fn with_no_derive_traits(self) -> Self {
        Self {
            no_derive_traits: true,
            ..self
        }
    }

    pub fn with_client_path(self, client_path: &'ast Path) -> Self {
        Self {
            client_path: Some(client_path),
            ..self
        }
    }
}

impl<'ast> ClientGenerator<'ast, IdlPath<'ast>> {
    pub fn from_idl_path(idl_path: &'ast Path) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            external_types: HashMap::new(),
            no_derive_traits: false,
            with_no_std: false,
            client_path: None,
            idl: IdlPath(idl_path),
        }
    }

    pub fn generate(self) -> Result<()> {
        let client_path = self.client_path.context("client path not set")?;
        let idl_path = self.idl.0;

        let path_str = idl_path.to_string_lossy();
        let idl = preprocess::preprocess(&path_str, &[&FsLoader, &GitLoader])
            .with_context(|| format!("Failed to open {} for reading", idl_path.display()))?;

        self.with_idl(&idl)
            .generate_to(client_path)
            .context("failed to generate client")?;
        Ok(())
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let idl_path = self.idl.0;

        let path_str = idl_path.to_string_lossy();
        let idl = preprocess::preprocess(&path_str, &[&FsLoader, &GitLoader])
            .with_context(|| format!("Failed to open {} for reading", idl_path.display()))?;

        self.with_idl(&idl)
            .generate_to(out_path)
            .context("failed to generate client")?;
        Ok(())
    }

    fn with_idl(self, idl: &'ast str) -> ClientGenerator<'ast, IdlString<'ast>> {
        ClientGenerator {
            sails_path: self.sails_path,
            mocks_feature_name: self.mocks_feature_name,
            external_types: self.external_types,
            no_derive_traits: self.no_derive_traits,
            with_no_std: self.with_no_std,
            client_path: self.client_path,
            idl: IdlString(idl),
        }
    }
}

impl<'ast> ClientGenerator<'ast, IdlString<'ast>> {
    pub fn from_idl(idl: &'ast str) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            external_types: HashMap::new(),
            no_derive_traits: false,
            with_no_std: false,
            client_path: None,
            idl: IdlString(idl),
        }
    }

    pub fn generate(self) -> Result<String> {
        let idl = self.idl.0;
        let sails_path = self.sails_path.unwrap_or(SAILS);
        let mut generator = RootGenerator::new(
            self.mocks_feature_name,
            sails_path,
            self.external_types,
            self.no_derive_traits,
        );
        let doc = parse_idl(idl).context("Failed to parse IDL")?;
        visitor::accept_idl_doc(&doc, &mut generator);

        let code = generator.finalize(self.with_no_std);

        // Check for parsing errors
        let code = pretty_with_rustfmt(&code);

        Ok(code)
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self.generate().context("failed to generate client")?;

        fs::write(out_path, code).with_context(|| {
            format!("Failed to write generated client to {}", out_path.display())
        })?;

        Ok(())
    }
}

// not using prettyplease since it's bad at reporting syntax errors and also removes comments
// TODO(holykol): Fallback if rustfmt is not in PATH would be nice
fn pretty_with_rustfmt(code: &str) -> String {
    use std::process::Command;
    let mut child = Command::new("rustfmt")
        .arg("--config")
        .arg("style_edition=2024")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt");

    let child_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    child_stdin
        .write_all(code.as_bytes())
        .expect("Failed to write to rustfmt");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for rustfmt");

    if !output.status.success() {
        panic!(
            "rustfmt failed with status: {}\n{}",
            output.status,
            String::from_utf8(output.stderr).expect("Failed to read rustfmt stderr")
        );
    }

    String::from_utf8(output.stdout).expect("Failed to read rustfmt output")
}

#[cfg(test)]
mod tests {
    use sails_idl_parser_v2::{FsLoader, GitLoader, preprocess};

    #[test]
    fn test_resolve_idl_from_path() {
        let path = "tests/idls/recursive_main.idl";
        let result =
            preprocess::preprocess(path, &[&FsLoader]).expect("Failed to resolve nested IDL");

        assert!(result.contains("service Leaf"));
        assert!(result.contains("service Middle"));
        assert!(result.contains("service Main"));
    }

    #[test]
    fn test_resolve_nested_idl() {
        let path = "tests/idls/nested/main.idl";
        let result =
            preprocess::preprocess(path, &[&FsLoader]).expect("Failed to resolve nested IDL");

        assert!(result.contains("service A"));
        assert!(result.contains("service B"));
        assert!(result.contains("service Main"));

        let common_count = result.matches("struct Common").count();
        assert_eq!(
            common_count, 1,
            "struct Common should be included only once, but found {}",
            common_count
        );
    }

    #[test]
    #[ignore]
    fn test_git_include_demo() {
        let path = "tests/idls/git_include/main.idl";
        let result = preprocess::preprocess(path, &[&FsLoader, &GitLoader])
            .expect("Failed to preprocess git include chain");

        let doc = sails_idl_parser_v2::parse_idl(&result)
            .expect("Failed to parse IDL from git include chain");

        let service_names: Vec<_> = doc.services.iter().map(|s| s.name.to_string()).collect();
        assert!(
            service_names.iter().any(|n| n.contains("PingPong")),
            "Expected PingPong service from demo_client.idl, got: {service_names:?}"
        );
        assert!(
            service_names.iter().any(|n| n.contains("Counter")),
            "Expected Counter service from demo_client.idl, got: {service_names:?}"
        );
    }
}
