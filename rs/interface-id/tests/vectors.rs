use std::{collections::BTreeMap, fs, path::PathBuf};

use sails_interface_id::{canonical::CanonicalDocument, compute_ids_from_document};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("vectors")
        .join("canonical")
}

#[test]
fn canonical_vectors_match_expected() {
    let manifest_path = vectors_dir().join("manifest.json");
    let manifest_contents =
        fs::read_to_string(&manifest_path).expect("failed to read canonical manifest");
    let manifest: BTreeMap<String, BTreeMap<String, String>> =
        serde_json::from_str(&manifest_contents).expect("invalid canonical manifest json");

    for (file, expected_services) in manifest {
        let doc_path = vectors_dir().join(&file);
        let canonical_str =
            fs::read_to_string(&doc_path).unwrap_or_else(|_| panic!("failed to read {file}"));
        let canonical =
            CanonicalDocument::from_json_str(&canonical_str).expect("vector canonical parse");
        for (service_name, expected_hex) in expected_services {
            let expected_id = u64::from_str_radix(expected_hex.trim_start_matches("0x"), 16)
                .expect("invalid hex interface id");

            let service = canonical
                .services()
                .get(&service_name)
                .unwrap_or_else(|| panic!("service {service_name} missing in {file}"))
                .clone();

            let mut single_services = BTreeMap::new();
            single_services.insert(service_name.clone(), service);

            let single_doc = CanonicalDocument::from_parts(
                canonical.canon_schema(),
                canonical.canon_version(),
                canonical.hash().clone(),
                single_services,
                canonical.types().clone(),
            );

            let actual_id = compute_ids_from_document(&single_doc);
            assert_eq!(
                actual_id, expected_id,
                "interface_id mismatch for {service_name} in {file}"
            );
        }
    }
}
