mod generator;
mod types;

use anyhow::{bail, Context, Result};
use generator::*;
use std::{env, fs, path::PathBuf, process};
use types::*;

fn main() -> Result<()> {
    let idl_json_path = match std::env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: client-gen <idl.json>");
            std::process::exit(1);
        }
    };

    let idl = fs::File::open(&idl_json_path)
        .with_context(|| format!("Failed to open {} for reading", idl_json_path.display()))?;

    let jd = &mut serde_json::Deserializer::from_reader(&idl);

    let idl: Idl = serde_path_to_error::deserialize(jd).context("deserialize IDL")?;

    let builder = IdlGenerator::new(idl_json_path);
    let buf = builder.generate(idl).context("failed to generate client")?;

    let buf = prettify(buf)?;

    print!("{}", buf);

    Ok(())
}

fn prettify(buf: String) -> Result<String> {
    let mut tmp_path = env::temp_dir();
    tmp_path.push(format!("client.{}.rs", rand::random::<u16>()));

    fs::write(&tmp_path, buf.as_bytes()).context("write temp file")?;

    // run rustfmt against temp file
    let status = process::Command::new("rustfmt")
        .arg(format!("{}", tmp_path.display()))
        .spawn()
        .context("failed spawn rustfmt. Make sure it's in your PATH")?
        .wait()
        .context("wait for rustfmt to finish")?;

    if !status.success() {
        bail!("rustfmt returned non-zero exit code. exiting");
    }

    let result = fs::read_to_string(&tmp_path).context("read resulting file")?;
    fs::remove_file(&tmp_path).context("remove temp file")?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full() {
        let json = include_str!("fixtures/this-that.json");
        let idl: Idl = serde_json::from_str(json).unwrap();

        let generator = IdlGenerator::new(PathBuf::from("test"));

        let generated = generator.generate(idl).unwrap();
        let generated = prettify(generated).unwrap();

        insta::assert_snapshot!(generated);
    }
}
