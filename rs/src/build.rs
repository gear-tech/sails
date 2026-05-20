//! Build-script helpers for Sails programs.

pub use gwasm_builder::build as build_wasm;

use gwasm_builder::{PreProcessor, PreProcessorTarget, WasmBuilder};
use sails_storage::{
    ACTOR_ID_U256_SLOT_SIZE, ACTOR_PAIR_U256_SLOT_SIZE, StaticLayout, StaticOpenAddressTable,
};
use std::{
    boxed::Box,
    collections::BTreeSet,
    env, eprintln,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process,
    string::{String, ToString},
    vec,
    vec::Vec,
};

const GENERATED_STATIC_STORAGE: &str = "sails_static_storage.rs";
const WASM_PAGE_SIZE: usize = 64 * 1024;

/// Maximum static WASM pages accepted by Gear.
pub const MAX_STATIC_MEMORY_PAGES: u32 = 32_768;

/// Static memory layout reserved by a Sails build script.
#[derive(Clone, Debug)]
pub struct StaticMemoryLayout {
    start_page: u32,
    tables: Vec<StaticTable>,
}

#[derive(Clone, Debug)]
struct StaticTable {
    name: String,
    slots: StaticTableSlots,
    kind: StaticTableKind,
}

#[derive(Clone, Debug)]
enum StaticTableSlots {
    Exact(usize),
    Log2(u8),
}

#[derive(Clone, Debug)]
enum StaticTableKind {
    SingleRegion { slot_size: usize, align: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedStaticMemoryLayout {
    start_page: u32,
    end_page: u32,
    tables: Vec<ResolvedStaticTable>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedStaticTable {
    name: String,
    base: usize,
    control_base: Option<usize>,
    slots_base: Option<usize>,
    slots: usize,
    log2_slots: Option<u8>,
    tiles: Option<usize>,
    log2_tiles: Option<u8>,
    slots_per_tile: Option<usize>,
    tile_bytes: Option<usize>,
    groups: Option<usize>,
    log2_groups: Option<u8>,
    group_pages: Option<usize>,
    log2_group_pages: Option<u8>,
    slots_per_group: Option<usize>,
    group_bytes: Option<usize>,
    data_offset: Option<usize>,
    mask: Option<usize>,
    control_bytes: Option<usize>,
    slots_bytes: Option<usize>,
    bytes: usize,
}

/// Errors returned by static-memory build helpers.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("static memory table name `{0}` is not a valid snake_case Rust identifier")]
    InvalidTableName(String),
    #[error("static memory table name `{0}` is reserved by Rust")]
    ReservedTableName(String),
    #[error("static memory table name `{0}` is duplicated")]
    DuplicateTableName(String),
    #[error("static memory layout overflows")]
    LayoutOverflow,
    #[error("static memory layout exceeds Gear limit of {MAX_STATIC_MEMORY_PAGES} WASM pages")]
    StaticMemoryLimitExceeded,
    #[error("static memory layout is invalid: {0}")]
    StorageLayout(sails_storage::TableError),
    #[error("OUT_DIR is not set for static memory source generation")]
    MissingOutDir(#[from] env::VarError),
    #[error("static memory build I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("WASM parse failed while patching static memory: {0}")]
    WasmParse(#[from] wasmparser::BinaryReaderError),
    #[error("WASM import cannot be round-tripped while patching static memory: {0}")]
    UnsupportedImport(String),
    #[error("WASM payload cannot be round-tripped while patching static memory")]
    UnsupportedWasmPayload,
    #[error("WASM module has no imported memory")]
    MemoryImportNotFound,
    #[error("WASM module has multiple imported memories")]
    MultipleMemoryImports,
    #[error("WASM memory uses custom page size {0:?}; Sails static memory requires 64 KiB pages")]
    UnsupportedMemoryPageSize(Option<u32>),
    #[error(
        "static memory start page {start_page} overlaps the original imported static pages {original_pages}"
    )]
    StaticMemoryOverlapsProgram {
        start_page: u32,
        original_pages: u64,
    },
    #[error(
        "static memory requires {required_pages} pages, exceeding imported memory maximum {maximum_pages}"
    )]
    StaticMemoryExceedsMemoryMaximum {
        required_pages: u64,
        maximum_pages: u64,
    },
}

/// Builds a WASM binary after reserving static memory pages.
///
/// This function exits the build script on error, matching
/// [`gwasm_builder::build`].
pub fn build_wasm_with_static_memory(layout: StaticMemoryLayout) -> Option<(PathBuf, PathBuf)> {
    match try_build_wasm_with_static_memory(layout) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}

/// Fallible variant of [`build_wasm_with_static_memory`].
pub fn try_build_wasm_with_static_memory(
    layout: StaticMemoryLayout,
) -> Result<Option<(PathBuf, PathBuf)>, Error> {
    let layout = layout.resolve()?;
    emit_static_storage(&layout)?;

    Ok(WasmBuilder::new()
        .exclude_features(vec!["std"])
        .with_pre_processor(Box::new(StaticMemoryPreProcessor { layout }))
        .build())
}

impl StaticMemoryLayout {
    /// Creates a layout that starts at `start_page`.
    pub fn new(start_page: u32) -> Self {
        Self {
            start_page,
            tables: Vec::new(),
        }
    }

    /// Reserves a named static open-addressed table.
    pub fn reserve_table<const KEY_SIZE: usize, const VALUE_SIZE: usize>(
        mut self,
        name: impl Into<String>,
        slots: usize,
    ) -> Self {
        self.tables.push(StaticTable {
            name: name.into(),
            slots: StaticTableSlots::Exact(slots),
            kind: StaticTableKind::SingleRegion {
                slot_size: StaticOpenAddressTable::<KEY_SIZE, VALUE_SIZE>::slot_size(),
                align: 1,
            },
        });
        self
    }

    /// Reserves a WAT-shaped `ActorId -> U256` static map.
    ///
    /// The reserved table uses `2^LOG2_SLOTS` slots. Each slot is exactly 64
    /// bytes: a 32-byte actor id followed by a 32-byte `U256` value.
    pub fn reserve_actor_u256_map<const LOG2_SLOTS: u8>(mut self, name: impl Into<String>) -> Self {
        self.tables.push(StaticTable {
            name: name.into(),
            slots: StaticTableSlots::Log2(LOG2_SLOTS),
            kind: StaticTableKind::SingleRegion {
                slot_size: ACTOR_ID_U256_SLOT_SIZE,
                align: 64,
            },
        });
        self
    }

    /// Reserves a `(ActorId, ActorId) -> U256` static map.
    ///
    /// The reserved table uses `2^LOG2_SLOTS` slots. Each slot is exactly 96
    /// bytes: owner actor id, spender actor id, and a `U256` value.
    pub fn reserve_actor_pair_u256_map<const LOG2_SLOTS: u8>(
        mut self,
        name: impl Into<String>,
    ) -> Self {
        self.tables.push(StaticTable {
            name: name.into(),
            slots: StaticTableSlots::Log2(LOG2_SLOTS),
            kind: StaticTableKind::SingleRegion {
                slot_size: ACTOR_PAIR_U256_SLOT_SIZE,
                align: 32,
            },
        });
        self
    }
    fn resolve(self) -> Result<ResolvedStaticMemoryLayout, Error> {
        let start_base = page_offset(self.start_page)?;
        let max_bytes = page_offset(MAX_STATIC_MEMORY_PAGES)?;
        let bytes = max_bytes
            .checked_sub(start_base)
            .ok_or(Error::StaticMemoryLimitExceeded)?;
        let mut layout = StaticLayout::new(start_base, bytes).map_err(Error::StorageLayout)?;
        let mut names = BTreeSet::new();
        let mut tables = Vec::with_capacity(self.tables.len());

        for table in self.tables {
            validate_table_name(&table.name)?;
            if !names.insert(table.name.clone()) {
                return Err(Error::DuplicateTableName(table.name));
            }

            let log2_slots = table.slots.log2();
            let slots = table.slots.resolve()?;
            let resolved = match table.kind {
                StaticTableKind::SingleRegion { slot_size, align } => {
                    let bytes = slots.checked_mul(slot_size).ok_or(Error::LayoutOverflow)?;
                    let region = layout
                        .reserve_aligned_bytes(bytes, align)
                        .map_err(Error::StorageLayout)?;
                    ResolvedStaticTable {
                        name: table.name,
                        base: region.base(),
                        control_base: None,
                        slots_base: None,
                        slots,
                        log2_slots,
                        tiles: None,
                        log2_tiles: None,
                        slots_per_tile: None,
                        tile_bytes: None,
                        groups: None,
                        log2_groups: None,
                        group_pages: None,
                        log2_group_pages: None,
                        slots_per_group: None,
                        group_bytes: None,
                        data_offset: None,
                        mask: log2_slots.map(|_| slots - 1),
                        control_bytes: None,
                        slots_bytes: None,
                        bytes: region.bytes(),
                    }
                }
            };
            tables.push(resolved);
        }

        let end_page = byte_end_to_page(layout.cursor())?;
        Ok(ResolvedStaticMemoryLayout {
            start_page: self.start_page,
            end_page,
            tables,
        })
    }
}

impl StaticTableSlots {
    fn log2(&self) -> Option<u8> {
        match *self {
            Self::Exact(_) => None,
            Self::Log2(log2_slots) => Some(log2_slots),
        }
    }

    fn resolve(&self) -> Result<usize, Error> {
        match *self {
            Self::Exact(slots) => Ok(slots),
            Self::Log2(log2_slots) => {
                let shift = u32::from(log2_slots);
                if shift > 32 || shift >= usize::BITS {
                    return Err(Error::LayoutOverflow);
                }

                1usize.checked_shl(shift).ok_or(Error::LayoutOverflow)
            }
        }
    }
}

struct StaticMemoryPreProcessor {
    layout: ResolvedStaticMemoryLayout,
}

impl PreProcessor for StaticMemoryPreProcessor {
    fn name(&self) -> &'static str {
        "sails_static_memory"
    }

    fn pre_process(
        &self,
        original: PathBuf,
    ) -> gwasm_builder::PreProcessorResult<Vec<(PreProcessorTarget, Vec<u8>)>> {
        let wasm = fs::read(original)?;
        let patched = patch_imported_memory(&wasm, self.layout.start_page, self.layout.end_page)?;
        Ok(vec![(PreProcessorTarget::Default, patched)])
    }
}

fn emit_static_storage(layout: &ResolvedStaticMemoryLayout) -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR")?;
    emit_static_storage_to_dir(layout, out_dir)
}

fn emit_static_storage_to_dir(
    layout: &ResolvedStaticMemoryLayout,
    out_dir: impl AsRef<Path>,
) -> Result<(), Error> {
    let mut source = String::new();
    writeln!(
        &mut source,
        "// @generated by sails_rs::build::build_wasm_with_static_memory"
    )
    .expect("writing to String cannot fail");
    writeln!(
        &mut source,
        "#[allow(dead_code)]\npub const STATIC_MEMORY_START_PAGE: u32 = {};",
        layout.start_page
    )
    .expect("writing to String cannot fail");
    writeln!(
        &mut source,
        "#[allow(dead_code)]\npub const STATIC_MEMORY_END_PAGE: u32 = {};",
        layout.end_page
    )
    .expect("writing to String cannot fail");

    for table in &layout.tables {
        let name = table.name.to_ascii_uppercase();
        writeln!(&mut source).expect("writing to String cannot fail");
        writeln!(
            &mut source,
            "#[allow(dead_code)]\npub const {name}_BASE: usize = {};",
            table.base
        )
        .expect("writing to String cannot fail");
        if let Some(control_base) = table.control_base {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_CONTROL_BASE: usize = {control_base};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(slots_base) = table.slots_base {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_SLOTS_BASE: usize = {slots_base};"
            )
            .expect("writing to String cannot fail");
        }
        writeln!(
            &mut source,
            "#[allow(dead_code)]\npub const {name}_SLOTS: usize = {};",
            table.slots
        )
        .expect("writing to String cannot fail");
        if let Some(log2_slots) = table.log2_slots {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_LOG2_SLOTS: u8 = {log2_slots};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(tiles) = table.tiles {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_TILES: usize = {tiles};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(log2_tiles) = table.log2_tiles {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_LOG2_TILES: u8 = {log2_tiles};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(slots_per_tile) = table.slots_per_tile {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_SLOTS_PER_TILE: usize = {slots_per_tile};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(tile_bytes) = table.tile_bytes {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_TILE_BYTES: usize = {tile_bytes};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(groups) = table.groups {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_GROUPS: usize = {groups};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(log2_groups) = table.log2_groups {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_LOG2_GROUPS: u8 = {log2_groups};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(group_pages) = table.group_pages {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_GROUP_PAGES: usize = {group_pages};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(log2_group_pages) = table.log2_group_pages {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_LOG2_GROUP_PAGES: u8 = {log2_group_pages};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(slots_per_group) = table.slots_per_group {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_SLOTS_PER_GROUP: usize = {slots_per_group};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(group_bytes) = table.group_bytes {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_GROUP_BYTES: usize = {group_bytes};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(data_offset) = table.data_offset {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_DATA_OFFSET: usize = {data_offset};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(mask) = table.mask {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_MASK: usize = {mask};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(control_bytes) = table.control_bytes {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_CONTROL_BYTES: usize = {control_bytes};"
            )
            .expect("writing to String cannot fail");
        }
        if let Some(slots_bytes) = table.slots_bytes {
            writeln!(
                &mut source,
                "#[allow(dead_code)]\npub const {name}_SLOTS_BYTES: usize = {slots_bytes};"
            )
            .expect("writing to String cannot fail");
        }
        writeln!(
            &mut source,
            "#[allow(dead_code)]\npub const {name}_BYTES: usize = {};",
            table.bytes
        )
        .expect("writing to String cannot fail");
    }

    fs::write(out_dir.as_ref().join(GENERATED_STATIC_STORAGE), source)?;
    Ok(())
}

fn patch_imported_memory(
    wasm: &[u8],
    start_page: u32,
    required_pages: u32,
) -> Result<Vec<u8>, Error> {
    let parser = wasmparser::Parser::new(0);
    let mut module = wasm_encoder::Module::new();
    let mut memory_import_seen = false;

    for payload in parser.parse_all(wasm) {
        match payload? {
            wasmparser::Payload::Version { .. } => {}
            wasmparser::Payload::ImportSection(imports) => {
                let mut import_section = wasm_encoder::ImportSection::new();

                for imports in imports {
                    match imports? {
                        wasmparser::Imports::Single(_, import) => append_import(
                            &mut import_section,
                            import.module,
                            import.name,
                            import.ty,
                            &mut memory_import_seen,
                            start_page,
                            required_pages,
                        )?,
                        wasmparser::Imports::Compact1 { module, items } => {
                            for item in items {
                                let item = item?;
                                append_import(
                                    &mut import_section,
                                    module,
                                    item.name,
                                    item.ty,
                                    &mut memory_import_seen,
                                    start_page,
                                    required_pages,
                                )?;
                            }
                        }
                        wasmparser::Imports::Compact2 { module, ty, names } => {
                            for name in names {
                                append_import(
                                    &mut import_section,
                                    module,
                                    name?,
                                    ty,
                                    &mut memory_import_seen,
                                    start_page,
                                    required_pages,
                                )?;
                            }
                        }
                    }
                }

                module.section(&import_section);
            }
            wasmparser::Payload::End(_) => break,
            other => {
                if let Some((id, range)) = other.as_section() {
                    module.section(&wasm_encoder::RawSection {
                        id,
                        data: &wasm[range],
                    });
                } else if !matches!(
                    other,
                    wasmparser::Payload::CodeSectionStart { .. }
                        | wasmparser::Payload::CodeSectionEntry { .. }
                ) {
                    return Err(Error::UnsupportedWasmPayload);
                }
            }
        }
    }

    if !memory_import_seen {
        return Err(Error::MemoryImportNotFound);
    }

    Ok(module.finish())
}

fn append_import(
    imports: &mut wasm_encoder::ImportSection,
    module: &str,
    name: &str,
    ty: wasmparser::TypeRef,
    memory_import_seen: &mut bool,
    start_page: u32,
    required_pages: u32,
) -> Result<(), Error> {
    let entity = match ty {
        wasmparser::TypeRef::Memory(memory) => {
            if *memory_import_seen {
                return Err(Error::MultipleMemoryImports);
            }
            *memory_import_seen = true;
            patched_memory_type(memory, start_page, required_pages)?
        }
        other => other
            .try_into()
            .map_err(|error: wasm_encoder::reencode::Error| {
                Error::UnsupportedImport(error.to_string())
            })?,
    };

    imports.import(module, name, entity);
    Ok(())
}

fn patched_memory_type(
    memory: wasmparser::MemoryType,
    start_page: u32,
    required_pages: u32,
) -> Result<wasm_encoder::EntityType, Error> {
    if memory.page_size_log2.is_some_and(|page| page != 16) {
        return Err(Error::UnsupportedMemoryPageSize(memory.page_size_log2));
    }

    if u64::from(start_page) < memory.initial {
        return Err(Error::StaticMemoryOverlapsProgram {
            start_page,
            original_pages: memory.initial,
        });
    }

    let minimum = u64::from(required_pages).max(memory.initial);
    if let Some(maximum) = memory.maximum
        && minimum > maximum
    {
        return Err(Error::StaticMemoryExceedsMemoryMaximum {
            required_pages: minimum,
            maximum_pages: maximum,
        });
    }

    Ok(wasm_encoder::EntityType::Memory(wasm_encoder::MemoryType {
        minimum,
        maximum: memory.maximum,
        memory64: memory.memory64,
        shared: memory.shared,
        page_size_log2: memory.page_size_log2,
    }))
}

fn page_offset(page: u32) -> Result<usize, Error> {
    (page as usize)
        .checked_mul(WASM_PAGE_SIZE)
        .ok_or(Error::LayoutOverflow)
}

fn byte_end_to_page(byte_end: usize) -> Result<u32, Error> {
    let pages = byte_end.div_ceil(WASM_PAGE_SIZE);
    if pages > MAX_STATIC_MEMORY_PAGES as usize {
        return Err(Error::StaticMemoryLimitExceeded);
    }
    u32::try_from(pages).map_err(|_| Error::StaticMemoryLimitExceeded)
}

fn validate_table_name(name: &str) -> Result<(), Error> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(Error::InvalidTableName(name.to_string()));
    };

    if !(first == '_' || first.is_ascii_lowercase()) {
        return Err(Error::InvalidTableName(name.to_string()));
    }

    if !chars.all(|ch| ch == '_' || ch.is_ascii_lowercase() || ch.is_ascii_digit()) {
        return Err(Error::InvalidTableName(name.to_string()));
    }

    if is_rust_keyword(name) {
        return Err(Error::ReservedTableName(name.to_string()));
    }

    Ok(())
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{EntityType, ImportSection, MemoryType, Module};

    fn module_with_memory(minimum: u64, maximum: Option<u64>) -> Vec<u8> {
        let mut imports = ImportSection::new();
        imports.import(
            "env",
            "memory",
            EntityType::Memory(MemoryType {
                minimum,
                maximum,
                memory64: false,
                shared: false,
                page_size_log2: None,
            }),
        );

        let mut module = Module::new();
        module.section(&imports);
        module.finish()
    }

    fn memory_minimum(wasm: &[u8]) -> u64 {
        for payload in wasmparser::Parser::new(0).parse_all(wasm) {
            let wasmparser::Payload::ImportSection(imports) = payload.unwrap() else {
                continue;
            };
            for imports in imports {
                match imports.unwrap() {
                    wasmparser::Imports::Single(_, import) => {
                        if let wasmparser::TypeRef::Memory(memory) = import.ty {
                            return memory.initial;
                        }
                    }
                    wasmparser::Imports::Compact1 { items, .. } => {
                        for item in items {
                            let item = item.unwrap();
                            if let wasmparser::TypeRef::Memory(memory) = item.ty {
                                return memory.initial;
                            }
                        }
                    }
                    wasmparser::Imports::Compact2 { ty, .. } => {
                        if let wasmparser::TypeRef::Memory(memory) = ty {
                            return memory.initial;
                        }
                    }
                }
            }
        }

        panic!("memory import not found")
    }

    #[test]
    fn resolves_layout_and_generates_constants() {
        let layout = StaticMemoryLayout::new(1024)
            .reserve_table::<32, 32>("balances", 2)
            .reserve_table::<64, 32>("allowances", 1)
            .resolve()
            .unwrap();

        assert_eq!(layout.start_page, 1024);
        assert_eq!(layout.end_page, 1025);
        assert_eq!(layout.tables[0].base, 1024 * WASM_PAGE_SIZE);
        assert_eq!(layout.tables[0].bytes, 2 * (1 + 32 + 32));
        assert_eq!(layout.tables[1].bytes, 1 + 64 + 32);

        let dir = tempfile::tempdir().unwrap();
        emit_static_storage_to_dir(&layout, dir.path()).unwrap();
        let generated = fs::read_to_string(dir.path().join(GENERATED_STATIC_STORAGE)).unwrap();
        assert!(generated.contains("pub const BALANCES_BASE: usize = 67108864;"));
        assert!(generated.contains("pub const ALLOWANCES_SLOTS: usize = 1;"));
    }

    #[test]
    fn resolves_specialized_static_maps_with_alignment_and_metadata() {
        let layout = StaticMemoryLayout::new(1024)
            .reserve_table::<1, 1>("prefix", 1)
            .reserve_actor_u256_map::<2>("fast_balances")
            .reserve_actor_pair_u256_map::<1>("fast_allowances")
            .resolve()
            .unwrap();

        assert_eq!(layout.tables[1].base % 64, 0);
        assert_eq!(layout.tables[1].slots, 4);
        assert_eq!(layout.tables[1].log2_slots, Some(2));
        assert_eq!(layout.tables[1].mask, Some(3));
        assert_eq!(layout.tables[1].bytes, 4 * ACTOR_ID_U256_SLOT_SIZE);
        assert_eq!(layout.tables[2].base % 32, 0);
        assert_eq!(layout.tables[2].slots, 2);
        assert_eq!(layout.tables[2].log2_slots, Some(1));
        assert_eq!(layout.tables[2].mask, Some(1));
        assert_eq!(layout.tables[2].bytes, 2 * ACTOR_PAIR_U256_SLOT_SIZE);

        let dir = tempfile::tempdir().unwrap();
        emit_static_storage_to_dir(&layout, dir.path()).unwrap();
        let generated = fs::read_to_string(dir.path().join(GENERATED_STATIC_STORAGE)).unwrap();
        assert!(generated.contains("pub const FAST_BALANCES_LOG2_SLOTS: u8 = 2;"));
        assert!(generated.contains("pub const FAST_BALANCES_MASK: usize = 3;"));
        assert!(generated.contains("pub const FAST_ALLOWANCES_LOG2_SLOTS: u8 = 1;"));
        assert!(generated.contains("pub const FAST_ALLOWANCES_MASK: usize = 1;"));
    }

    #[test]
    fn rejects_invalid_duplicate_and_reserved_names() {
        assert!(matches!(
            StaticMemoryLayout::new(1)
                .reserve_table::<1, 1>("bad-name", 1)
                .resolve(),
            Err(Error::InvalidTableName(_))
        ));
        assert!(matches!(
            StaticMemoryLayout::new(1)
                .reserve_table::<1, 1>("balances", 1)
                .reserve_table::<1, 1>("balances", 1)
                .resolve(),
            Err(Error::DuplicateTableName(_))
        ));
        assert!(matches!(
            StaticMemoryLayout::new(1)
                .reserve_table::<1, 1>("type", 1)
                .resolve(),
            Err(Error::ReservedTableName(_))
        ));
    }

    #[test]
    fn rejects_layout_overflow_and_page_limit() {
        assert!(matches!(
            StaticMemoryLayout::new(MAX_STATIC_MEMORY_PAGES)
                .reserve_table::<1, 1>("balances", 1)
                .resolve(),
            Err(Error::StorageLayout(_))
        ));
        assert!(matches!(
            StaticMemoryLayout::new(1)
                .reserve_table::<1, 1>("balances", usize::MAX)
                .resolve(),
            Err(Error::LayoutOverflow)
        ));
    }

    #[test]
    fn patches_memory_import_minimum() {
        let wasm = module_with_memory(1, None);
        let patched = patch_imported_memory(&wasm, 8, 10).unwrap();

        assert_eq!(memory_minimum(&patched), 10);
    }

    #[test]
    fn preserves_higher_existing_memory_minimum() {
        let wasm = module_with_memory(12, None);
        let patched = patch_imported_memory(&wasm, 12, 10).unwrap();

        assert_eq!(memory_minimum(&patched), 12);
    }

    #[test]
    fn rejects_overlap_with_original_static_pages() {
        let wasm = module_with_memory(12, None);
        let error = patch_imported_memory(&wasm, 8, 20).unwrap_err();

        assert!(matches!(
            error,
            Error::StaticMemoryOverlapsProgram {
                start_page: 8,
                original_pages: 12,
            }
        ));
    }

    #[test]
    fn rejects_memory_maximum_below_required_pages() {
        let wasm = module_with_memory(1, Some(4));
        let error = patch_imported_memory(&wasm, 2, 8).unwrap_err();

        assert!(matches!(
            error,
            Error::StaticMemoryExceedsMemoryMaximum {
                required_pages: 8,
                maximum_pages: 4,
            }
        ));
    }

    #[test]
    fn rejects_missing_memory_import() {
        let wasm = Module::new().finish();
        let error = patch_imported_memory(&wasm, 1, 2).unwrap_err();

        assert!(matches!(error, Error::MemoryImportNotFound));
    }
}
