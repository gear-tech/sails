use super::*;
use ::gtest::BlockRunResult;
use sails_idl_meta::{SailsMessageHeader, ServiceMeta};
use std::collections::{BTreeMap, HashMap};

/// Registry mapping (InterfaceId, entry_id) pairs to human-readable method names.
///
/// Built via the builder pattern using `register_service` for each service.
#[derive(Default, Clone)]
pub struct MethodRegistry {
    /// Key: (interface_id as u64, entry_id) -> "ServiceName::method_name"
    methods: HashMap<(u64, u16), String>,
    /// Key: interface_id as u64 -> service name
    services: HashMap<u64, String>,
}

impl MethodRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register all methods from a `ServiceMeta` implementation.
    ///
    /// `service_name` is the human-readable name (e.g., `"Counter"`).
    pub fn register_service<S: ServiceMeta>(mut self, service_name: &str) -> Self {
        let iid = S::INTERFACE_ID.as_u64();
        self.services.insert(iid, service_name.to_string());
        for method in S::METHODS {
            self.methods.insert(
                (iid, method.entry_id),
                format!("{}::{}", service_name, method.name),
            );
        }
        self
    }

    /// Resolve interface_id + entry_id to a human-readable method name.
    pub fn resolve(&self, interface_id: InterfaceId, entry_id: u16) -> Option<&str> {
        self.methods
            .get(&(interface_id.as_u64(), entry_id))
            .map(|s| s.as_str())
    }

    /// Resolve just the service name from an interface_id.
    pub fn resolve_service(&self, interface_id: InterfaceId) -> Option<&str> {
        self.services
            .get(&interface_id.as_u64())
            .map(|s| s.as_str())
    }
}

/// Decoded method identification from a message payload.
#[derive(Debug, Clone)]
pub enum MethodInfo {
    /// Successfully decoded a Sails header.
    Sails {
        interface_id: InterfaceId,
        entry_id: u16,
        resolved_name: Option<String>,
    },
    /// Not a Sails message (no GM magic, too short, etc.)
    Raw,
}

/// A single node in the gas trace tree.
#[derive(Debug, Clone)]
pub struct GasTraceNode {
    pub message_id: MessageId,
    pub source: ActorId,
    pub destination: ActorId,
    pub gas: Option<u64>,
    pub method: MethodInfo,
    pub is_reply: bool,
    pub reply_code: Option<gear_core_errors::ReplyCode>,
    pub is_event: bool,
    pub children: Vec<GasTraceNode>,
}

/// A fully reconstructed message trace tree with gas annotations.
#[derive(Debug, Clone)]
pub struct GasTraceTree {
    pub roots: Vec<GasTraceNode>,
    pub total_gas: u64,
    pub total_messages: usize,
    pub max_depth: usize,
    actor_names: HashMap<ActorId, String>,
}

/// Builder for constructing gas trace trees from `BlockRunResult`s.
///
/// # Limitations
///
/// The tree is built from `reply_to` links, which connect replies to their
/// originating request. For single-program tests (user -> program -> reply),
/// this produces a complete call tree. For cross-program calls (A -> B -> C),
/// outbound sub-calls appear as separate root messages because gtest's `CoreLog`
/// does not expose parent-message causality.
pub struct GasTrace<'a> {
    blocks: Vec<&'a BlockRunResult>,
    registry: Option<&'a MethodRegistry>,
    actor_names: HashMap<ActorId, String>,
}

impl<'a> GasTrace<'a> {
    /// Create a new trace from a single block result.
    pub fn new(block: &'a BlockRunResult) -> Self {
        Self {
            blocks: vec![block],
            registry: None,
            actor_names: HashMap::new(),
        }
    }

    /// Create a trace spanning multiple block results.
    pub fn from_blocks(blocks: impl IntoIterator<Item = &'a BlockRunResult>) -> Self {
        Self {
            blocks: blocks.into_iter().collect(),
            registry: None,
            actor_names: HashMap::new(),
        }
    }

    /// Attach a method registry for human-readable method names.
    pub fn with_registry(mut self, registry: &'a MethodRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Register a human-readable name for an actor.
    pub fn with_actor_name(mut self, actor_id: ActorId, name: impl Into<String>) -> Self {
        self.actor_names.insert(actor_id, name.into());
        self
    }

    /// Build the trace tree.
    pub fn build(&self) -> GasTraceTree {
        // 1. Merge all logs and gas_burned from all blocks
        let mut all_logs = Vec::new();
        let mut all_gas: BTreeMap<MessageId, u64> = BTreeMap::new();

        for block in &self.blocks {
            for entry in block.log().iter() {
                all_logs.push(entry);
            }
            for (&msg_id, &gas) in &block.gas_burned {
                all_gas.insert(msg_id, gas);
            }
        }

        // 2. Index replies: reply_to -> list of log entries that are replies to it
        let mut replies_by_parent: HashMap<MessageId, Vec<&::gtest::CoreLog>> = HashMap::new();
        let mut roots = Vec::new();
        let mut events = Vec::new();

        for entry in &all_logs {
            if entry.destination() == ActorId::zero() {
                events.push(*entry);
            } else if let Some(parent_id) = entry.reply_to() {
                replies_by_parent
                    .entry(parent_id)
                    .or_default()
                    .push(entry);
            } else {
                roots.push(*entry);
            }
        }

        // 3. Build tree recursively
        let mut root_nodes: Vec<GasTraceNode> = roots
            .iter()
            .map(|entry| self.build_node(entry, &replies_by_parent, &all_gas))
            .collect();

        // 4. Collect orphaned replies (reply_to target not in any block)
        let root_ids: std::collections::HashSet<MessageId> =
            roots.iter().map(|e| e.id()).collect();
        for (parent_id, replies) in &replies_by_parent {
            if !root_ids.contains(parent_id)
                && !replies_by_parent.values().any(|v| v.iter().any(|r| r.id() == *parent_id))
            {
                // This parent was never seen as a root or as another reply's child
                for reply in replies {
                    root_nodes.push(GasTraceNode {
                        message_id: reply.id(),
                        source: reply.source(),
                        destination: reply.destination(),
                        gas: all_gas.get(&reply.id()).copied(),
                        method: self.decode_method(reply.payload()),
                        is_reply: true,
                        reply_code: reply.reply_code(),
                        is_event: false,
                        children: Vec::new(),
                    });
                }
            }
        }

        // 5. Events as top-level entries
        for entry in &events {
            root_nodes.push(GasTraceNode {
                message_id: entry.id(),
                source: entry.source(),
                destination: entry.destination(),
                gas: all_gas.get(&entry.id()).copied(),
                method: self.decode_method(entry.payload()),
                is_reply: false,
                reply_code: None,
                is_event: true,
                children: Vec::new(),
            });
        }

        // 6. Compute totals from tree nodes, not raw maps
        let total_gas = self.sum_gas(&root_nodes);
        let total_messages = all_logs.len();
        let max_depth = root_nodes.iter().map(|n| self.tree_depth(n)).max().unwrap_or(0);

        GasTraceTree {
            roots: root_nodes,
            total_gas,
            total_messages,
            max_depth,
            actor_names: self.actor_names.clone(),
        }
    }

    /// Build and print to stdout.
    pub fn print(&self) {
        std::println!("{}", self.build());
    }

    /// Build and return as a formatted string.
    pub fn to_string_pretty(&self) -> String {
        format!("{}", self.build())
    }

    fn build_node(
        &self,
        entry: &::gtest::CoreLog,
        replies_by_parent: &HashMap<MessageId, Vec<&::gtest::CoreLog>>,
        all_gas: &BTreeMap<MessageId, u64>,
    ) -> GasTraceNode {
        let children = replies_by_parent
            .get(&entry.id())
            .map(|replies| {
                replies
                    .iter()
                    .map(|reply| self.build_node(reply, replies_by_parent, all_gas))
                    .collect()
            })
            .unwrap_or_default();

        GasTraceNode {
            message_id: entry.id(),
            source: entry.source(),
            destination: entry.destination(),
            gas: all_gas.get(&entry.id()).copied(),
            method: self.decode_method(entry.payload()),
            is_reply: entry.reply_to().is_some(),
            reply_code: entry.reply_code(),
            is_event: false,
            children,
        }
    }

    fn decode_method(&self, payload: &[u8]) -> MethodInfo {
        match SailsMessageHeader::try_from_bytes(payload) {
            Ok(header) => {
                let interface_id = header.interface_id();
                let entry_id = header.entry_id();
                let resolved_name = self
                    .registry
                    .and_then(|r| r.resolve(interface_id, entry_id))
                    .map(|s| s.to_string());
                MethodInfo::Sails {
                    interface_id,
                    entry_id,
                    resolved_name,
                }
            }
            Err(_) => MethodInfo::Raw,
        }
    }

    fn sum_gas(&self, nodes: &[GasTraceNode]) -> u64 {
        nodes
            .iter()
            .map(|n| n.gas.unwrap_or(0) + self.sum_gas(&n.children))
            .sum()
    }

    fn tree_depth(&self, node: &GasTraceNode) -> usize {
        if node.children.is_empty() {
            0
        } else {
            1 + node
                .children
                .iter()
                .map(|c| self.tree_depth(c))
                .max()
                .unwrap_or(0)
        }
    }
}

impl std::fmt::Display for GasTraceTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.roots.is_empty() {
            return write!(f, "(empty trace)");
        }

        for root in &self.roots {
            format_node(f, root, "", true, &self.actor_names)?;
        }
        write!(
            f,
            "Total: {} gas | {} messages | depth {}",
            format_gas(self.total_gas),
            self.total_messages,
            self.max_depth,
        )
    }
}

fn format_node(
    f: &mut std::fmt::Formatter<'_>,
    node: &GasTraceNode,
    prefix: &str,
    is_last: bool,
    actor_names: &HashMap<ActorId, String>,
) -> std::fmt::Result {
    let connector = if prefix.is_empty() {
        ""
    } else if is_last {
        "`-- "
    } else {
        "+-- "
    };

    // Message ID (first 4 bytes)
    let msg_id_hex = format!("{:.4}", node.message_id);

    if node.is_event {
        let source_name = format_actor(node.source, actor_names);
        writeln!(f, "{prefix}{connector}[event] {source_name}")?;
    } else if node.is_reply {
        let code_str = match node.reply_code {
            Some(gear_core_errors::ReplyCode::Success(_)) => "Ok",
            Some(gear_core_errors::ReplyCode::Error(ref reason)) => {
                // Use a static string for common cases
                match reason {
                    gear_core_errors::ErrorReplyReason::Execution(e) => match e {
                        gear_core_errors::SimpleExecutionError::RanOutOfGas => {
                            "Err(RanOutOfGas)"
                        }
                        _ => "Err(Execution)",
                    },
                    _ => "Err",
                }
            }
            Some(gear_core_errors::ReplyCode::Unsupported) => "Unsupported",
            None => "?",
        };
        let gas_str = format_gas_opt(node.gas);
        writeln!(f, "{prefix}{connector}[{msg_id_hex}] [reply] {code_str}  {gas_str}")?;
    } else {
        let source_name = format_actor(node.source, actor_names);
        let dest_name = format_actor(node.destination, actor_names);
        let method_str = format_method(&node.method);
        let gas_str = format_gas_opt(node.gas);
        writeln!(
            f,
            "{prefix}{connector}[{msg_id_hex}] {source_name} -> {dest_name}::{method_str}  {gas_str}"
        )?;
    }

    // Children
    let child_prefix = if prefix.is_empty() {
        "  ".to_string()
    } else if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}|   ")
    };

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        format_node(f, child, &child_prefix, child_is_last, actor_names)?;
    }

    Ok(())
}

fn format_actor(actor_id: ActorId, actor_names: &HashMap<ActorId, String>) -> String {
    if let Some(name) = actor_names.get(&actor_id) {
        name.clone()
    } else {
        format!("{:.4}", actor_id)
    }
}

fn format_method(method: &MethodInfo) -> String {
    match method {
        MethodInfo::Sails {
            resolved_name: Some(name),
            ..
        } => name.clone(),
        MethodInfo::Sails {
            interface_id,
            entry_id,
            ..
        } => format!("{interface_id}#{entry_id}"),
        MethodInfo::Raw => "[raw]".to_string(),
    }
}

fn format_gas(gas: u64) -> String {
    if gas == 0 {
        return "0".to_string();
    }

    let s = gas.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn format_gas_opt(gas: Option<u64>) -> String {
    match gas {
        Some(g) => format!("{} gas", format_gas(g)),
        None => "- gas".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gear_core::message::{MessageDetails, ReplyDetails, StoredMessage};
    use gprimitives::{ActorId, MessageId};

    fn make_log(
        id: MessageId,
        source: ActorId,
        destination: ActorId,
        payload: Vec<u8>,
        reply_to: Option<MessageId>,
        reply_code: Option<gear_core_errors::ReplyCode>,
    ) -> ::gtest::CoreLog {
        let details = match (reply_to, reply_code) {
            (Some(reply_id), Some(code)) => {
                Some(MessageDetails::Reply(ReplyDetails::new(reply_id, code)))
            }
            _ => None,
        };
        let stored = StoredMessage::new(
            id,
            source,
            destination,
            payload.try_into().expect("payload too large"),
            0,
            details,
        );
        ::gtest::CoreLog::from(stored)
    }

    fn make_sails_payload(interface_id: InterfaceId, entry_id: u16, route_id: u8) -> Vec<u8> {
        SailsMessageHeader::v1(interface_id, entry_id, route_id).to_bytes()
    }

    fn make_block(
        logs: Vec<::gtest::CoreLog>,
        gas_burned: Vec<(MessageId, u64)>,
    ) -> BlockRunResult {
        BlockRunResult {
            log: logs,
            gas_burned: gas_burned.into_iter().collect(),
            ..Default::default()
        }
    }

    // --- MethodRegistry tests ---

    #[test]
    fn registry_register_and_resolve() {
        // We can't easily create a real ServiceMeta impl in tests,
        // so test the HashMap-based resolve directly.
        let mut registry = MethodRegistry::new();
        let iid = InterfaceId::from_u64(42);
        registry
            .methods
            .insert((iid.as_u64(), 0), "Counter::increment".to_string());
        registry
            .services
            .insert(iid.as_u64(), "Counter".to_string());

        assert_eq!(registry.resolve(iid, 0), Some("Counter::increment"));
        assert_eq!(registry.resolve_service(iid), Some("Counter"));
    }

    #[test]
    fn registry_resolve_unknown() {
        let registry = MethodRegistry::new();
        let iid = InterfaceId::from_u64(999);
        assert_eq!(registry.resolve(iid, 0), None);
        assert_eq!(registry.resolve_service(iid), None);
    }

    #[test]
    fn registry_empty_service() {
        // Empty registry still works, just resolves nothing
        let registry = MethodRegistry::new();
        assert_eq!(registry.resolve(InterfaceId::zero(), 0), None);
    }

    #[test]
    fn registry_duplicate_registration() {
        let mut registry = MethodRegistry::new();
        let iid = InterfaceId::from_u64(1);
        registry
            .methods
            .insert((iid.as_u64(), 0), "First::method".to_string());
        registry
            .methods
            .insert((iid.as_u64(), 0), "Second::method".to_string());
        // Last write wins
        assert_eq!(registry.resolve(iid, 0), Some("Second::method"));
    }

    // --- Tree reconstruction tests ---

    #[test]
    fn single_block_root_and_reply() {
        let user = ActorId::from(1u64);
        let program = ActorId::from(2u64);
        let msg_id = MessageId::from(100u64);
        let reply_id = MessageId::from(101u64);

        let payload = make_sails_payload(InterfaceId::from_u64(42), 0, 1);
        let root_log = make_log(msg_id, user, program, payload, None, None);
        let reply_log = make_log(
            reply_id,
            program,
            user,
            vec![],
            Some(msg_id),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        let block = make_block(vec![root_log, reply_log], vec![(msg_id, 5000)]);
        let tree = GasTrace::new(&block).build();

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].message_id, msg_id);
        assert_eq!(tree.roots[0].children.len(), 1);
        assert!(tree.roots[0].children[0].is_reply);
        assert_eq!(tree.total_messages, 2);
        assert_eq!(tree.total_gas, 5000);
    }

    #[test]
    fn multi_block_merge() {
        let user = ActorId::from(1u64);
        let program = ActorId::from(2u64);
        let msg_id = MessageId::from(200u64);
        let reply_id = MessageId::from(201u64);

        let payload = make_sails_payload(InterfaceId::from_u64(1), 0, 1);
        let root_log = make_log(msg_id, user, program, payload, None, None);
        let reply_log = make_log(
            reply_id,
            program,
            user,
            vec![],
            Some(msg_id),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        let block1 = make_block(vec![root_log], vec![(msg_id, 3000)]);
        let block2 = make_block(vec![reply_log], vec![(reply_id, 200)]);

        let tree = GasTrace::from_blocks([&block1, &block2]).build();

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].children.len(), 1);
        assert_eq!(tree.total_gas, 3200);
        assert_eq!(tree.total_messages, 2);
    }

    #[test]
    fn non_sails_payload_falls_back_to_raw() {
        let msg_id = MessageId::from(300u64);
        let log = make_log(
            msg_id,
            ActorId::from(1u64),
            ActorId::from(2u64),
            vec![0x00, 0x01, 0x02],
            None,
            None,
        );

        let block = make_block(vec![log], vec![(msg_id, 1000)]);
        let tree = GasTrace::new(&block).build();

        assert!(matches!(tree.roots[0].method, MethodInfo::Raw));
    }

    #[test]
    fn events_as_top_level() {
        let program = ActorId::from(2u64);
        let msg_id = MessageId::from(400u64);
        let event_id = MessageId::from(401u64);

        let root_log = make_log(
            msg_id,
            ActorId::from(1u64),
            program,
            make_sails_payload(InterfaceId::from_u64(1), 0, 1),
            None,
            None,
        );
        let event_log = make_log(
            event_id,
            program,
            ActorId::zero(),
            vec![0xAB, 0xCD],
            None,
            None,
        );

        let block = make_block(vec![root_log, event_log], vec![(msg_id, 2000)]);
        let tree = GasTrace::new(&block).build();

        // Root message + event = 2 top-level entries
        assert_eq!(tree.roots.len(), 2);
        assert!(!tree.roots[0].is_event);
        assert!(tree.roots[1].is_event);
    }

    #[test]
    fn orphaned_reply_as_top_level() {
        let reply_id = MessageId::from(501u64);
        let missing_parent = MessageId::from(500u64);

        let reply_log = make_log(
            reply_id,
            ActorId::from(2u64),
            ActorId::from(1u64),
            vec![],
            Some(missing_parent),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        let block = make_block(vec![reply_log], vec![(reply_id, 100)]);
        let tree = GasTrace::new(&block).build();

        // Orphaned reply: its parent (msg 500) isn't in the block.
        // The reply is surfaced as a top-level node so it's not silently lost.
        assert_eq!(tree.roots.len(), 1);
        assert!(tree.roots[0].is_reply);
        assert_eq!(tree.total_messages, 1);
        assert_eq!(tree.total_gas, 100);
    }

    #[test]
    fn empty_block_produces_empty_tree() {
        let block = make_block(vec![], vec![]);
        let tree = GasTrace::new(&block).build();

        assert!(tree.roots.is_empty());
        assert_eq!(tree.total_gas, 0);
        assert_eq!(tree.total_messages, 0);
        assert_eq!(tree.max_depth, 0);
    }

    // --- Display tests ---

    #[test]
    fn display_single_root_with_reply() {
        let user = ActorId::from(1u64);
        let program = ActorId::from(2u64);
        let msg_id = MessageId::from(100u64);
        let reply_id = MessageId::from(101u64);

        let payload = make_sails_payload(InterfaceId::from_u64(42), 0, 1);
        let root_log = make_log(msg_id, user, program, payload, None, None);
        let reply_log = make_log(
            reply_id,
            program,
            user,
            vec![],
            Some(msg_id),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        let block = make_block(
            vec![root_log, reply_log],
            vec![(msg_id, 5000), (reply_id, 200)],
        );

        let output = GasTrace::new(&block)
            .with_actor_name(user, "alice")
            .with_actor_name(program, "MyProgram")
            .to_string_pretty();

        assert!(output.contains("alice"));
        assert!(output.contains("MyProgram"));
        assert!(output.contains("[reply] Ok"));
        assert!(output.contains("5,200 gas"));
    }

    #[test]
    fn display_deep_nesting() {
        let a = ActorId::from(1u64);
        let b = ActorId::from(2u64);
        let id1 = MessageId::from(1u64);
        let id2 = MessageId::from(2u64);
        let id3 = MessageId::from(3u64);

        let payload = make_sails_payload(InterfaceId::from_u64(1), 0, 1);
        let log1 = make_log(id1, a, b, payload.clone(), None, None);
        let log2 = make_log(
            id2,
            b,
            a,
            vec![],
            Some(id1),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );
        // A third reply nested under id2 (unusual but tests depth)
        let log3 = make_log(
            id3,
            a,
            b,
            vec![],
            Some(id2),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        let block = make_block(
            vec![log1, log2, log3],
            vec![(id1, 1000), (id2, 500), (id3, 100)],
        );
        let tree = GasTrace::new(&block).build();

        assert_eq!(tree.max_depth, 2);
        let output = format!("{tree}");
        assert!(output.contains("depth 2"));
    }

    #[test]
    fn display_missing_gas() {
        let msg_id = MessageId::from(600u64);
        let log = make_log(
            msg_id,
            ActorId::from(1u64),
            ActorId::from(2u64),
            make_sails_payload(InterfaceId::from_u64(1), 0, 1),
            None,
            None,
        );

        // No gas_burned entry for this message
        let block = make_block(vec![log], vec![]);
        let output = GasTrace::new(&block).to_string_pretty();

        assert!(output.contains("- gas"));
    }

    #[test]
    fn display_empty_tree() {
        let block = make_block(vec![], vec![]);
        let output = GasTrace::new(&block).to_string_pretty();

        assert_eq!(output, "(empty trace)");
    }

    // --- Formatting helpers ---

    #[test]
    fn demo_output() {
        let alice = ActorId::from(1u64);
        let program_a = ActorId::from(2u64);
        let program_b = ActorId::from(3u64);

        let iid_counter = InterfaceId::from_u64(0xAAAA_BBBB_CCCC_0001);
        let iid_storage = InterfaceId::from_u64(0xDDDD_EEEE_FFFF_0002);

        // alice -> ProgramA::Counter::increment (root call)
        let msg1 = MessageId::from(0x1001u64);
        let log1 = make_log(
            msg1,
            alice,
            program_a,
            make_sails_payload(iid_counter, 0, 1),
            None,
            None,
        );

        // ProgramA -> ProgramB::Storage::write (cross-program, separate root)
        let msg2 = MessageId::from(0x2002u64);
        let log2 = make_log(
            msg2,
            program_a,
            program_b,
            make_sails_payload(iid_storage, 1, 1),
            None,
            None,
        );

        // ProgramB replies to ProgramA
        let reply2 = MessageId::from(0x2003u64);
        let log_reply2 = make_log(
            reply2,
            program_b,
            program_a,
            vec![],
            Some(msg2),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        // ProgramA replies to alice
        let reply1 = MessageId::from(0x1002u64);
        let log_reply1 = make_log(
            reply1,
            program_a,
            alice,
            vec![],
            Some(msg1),
            Some(gear_core_errors::ReplyCode::Success(
                gear_core_errors::SuccessReplyReason::Manual,
            )),
        );

        // Event emitted by ProgramA
        let event_id = MessageId::from(0x3001u64);
        let log_event = make_log(
            event_id,
            program_a,
            ActorId::zero(),
            vec![0xAB, 0xCD],
            None,
            None,
        );

        let block = make_block(
            vec![log1, log2, log_reply2, log_reply1, log_event],
            vec![
                (msg1, 12_400),
                (msg2, 8_200),
                (reply2, 200),
                (reply1, 150),
            ],
        );

        let mut registry = MethodRegistry::new();
        registry.methods.insert(
            (iid_counter.as_u64(), 0),
            "Counter::increment".to_string(),
        );
        registry
            .services
            .insert(iid_counter.as_u64(), "Counter".to_string());
        registry
            .methods
            .insert((iid_storage.as_u64(), 1), "Storage::write".to_string());
        registry
            .services
            .insert(iid_storage.as_u64(), "Storage".to_string());

        let output = GasTrace::new(&block)
            .with_registry(&registry)
            .with_actor_name(alice, "alice")
            .with_actor_name(program_a, "ProgramA")
            .with_actor_name(program_b, "ProgramB")
            .to_string_pretty();

        assert!(output.contains("Counter::increment"));
        assert!(output.contains("Storage::write"));
        assert!(output.contains("alice"));
    }

    #[test]
    fn format_gas_with_separators() {
        assert_eq!(format_gas(0), "0");
        assert_eq!(format_gas(999), "999");
        assert_eq!(format_gas(1000), "1,000");
        assert_eq!(format_gas(1_000_000), "1,000,000");
        assert_eq!(format_gas(12_345), "12,345");
    }
}
