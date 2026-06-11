use crate::{Cfg, EdgeKind, Result};

pub fn branch_with_join() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let check = cfg.add_node(148.0, 52.0)?;
    let fast = cfg.add_node(124.0, 48.0)?;
    let slow = cfg.add_node(140.0, 48.0)?;
    let merge = cfg.add_node(124.0, 48.0)?;
    let exit = cfg.add_node(96.0, 44.0)?;

    cfg.add_edge(entry, check, EdgeKind::Default)?;
    cfg.add_edge(check, fast, EdgeKind::True)?;
    cfg.add_edge(check, slow, EdgeKind::False)?;
    cfg.add_edge(fast, merge, EdgeKind::Default)?;
    cfg.add_edge(slow, merge, EdgeKind::Default)?;
    cfg.add_edge(merge, exit, EdgeKind::Default)?;

    Ok(cfg)
}

pub fn counted_loop() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let init = cfg.add_node(92.0, 44.0)?;
    let header = cfg.add_node(112.0, 48.0)?;
    let body = cfg.add_node(128.0, 48.0)?;
    let latch = cfg.add_node(96.0, 44.0)?;
    let exit = cfg.add_node(116.0, 44.0)?;

    cfg.add_edge(entry, init, EdgeKind::Default)?;
    cfg.add_edge(init, header, EdgeKind::Default)?;
    cfg.add_edge(header, body, EdgeKind::True)?;
    cfg.add_edge(body, latch, EdgeKind::Default)?;
    cfg.add_edge(latch, header, EdgeKind::Default)?;
    cfg.add_edge(header, exit, EdgeKind::False)?;

    Ok(cfg)
}

pub fn nested_conditionals() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let parse = cfg.add_node(120.0, 48.0)?;
    let valid = cfg.add_node(96.0, 48.0)?;
    let privileged = cfg.add_node(128.0, 48.0)?;
    let admin = cfg.add_node(112.0, 48.0)?;
    let user = cfg.add_node(108.0, 48.0)?;
    let reject = cfg.add_node(96.0, 44.0)?;
    let audit = cfg.add_node(96.0, 44.0)?;
    let exit = cfg.add_node(96.0, 44.0)?;

    cfg.add_edge(entry, parse, EdgeKind::Default)?;
    cfg.add_edge(parse, valid, EdgeKind::Default)?;
    cfg.add_edge(valid, privileged, EdgeKind::True)?;
    cfg.add_edge(valid, reject, EdgeKind::False)?;
    cfg.add_edge(privileged, admin, EdgeKind::True)?;
    cfg.add_edge(privileged, user, EdgeKind::False)?;
    cfg.add_edge(admin, audit, EdgeKind::Default)?;
    cfg.add_edge(user, audit, EdgeKind::Default)?;
    cfg.add_edge(reject, audit, EdgeKind::Default)?;
    cfg.add_edge(audit, exit, EdgeKind::Default)?;

    Ok(cfg)
}

pub fn switch_dispatch() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let decode = cfg.add_node(140.0, 48.0)?;
    let add = cfg.add_node(88.0, 44.0)?;
    let sub = cfg.add_node(88.0, 44.0)?;
    let mul = cfg.add_node(88.0, 44.0)?;
    let fallback = cfg.add_node(88.0, 44.0)?;
    let normalize = cfg.add_node(148.0, 48.0)?;
    let exit = cfg.add_node(96.0, 44.0)?;

    cfg.add_edge(entry, decode, EdgeKind::Default)?;
    cfg.add_edge(decode, add, EdgeKind::Default)?;
    cfg.add_edge(decode, sub, EdgeKind::Default)?;
    cfg.add_edge(decode, mul, EdgeKind::Default)?;
    cfg.add_edge(decode, fallback, EdgeKind::Default)?;
    cfg.add_edge(add, normalize, EdgeKind::Default)?;
    cfg.add_edge(sub, normalize, EdgeKind::Default)?;
    cfg.add_edge(mul, normalize, EdgeKind::Default)?;
    cfg.add_edge(fallback, normalize, EdgeKind::Default)?;
    cfg.add_edge(normalize, exit, EdgeKind::Default)?;

    Ok(cfg)
}

pub fn retry_with_cleanup() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let acquire = cfg.add_node(104.0, 44.0)?;
    let call = cfg.add_node(124.0, 48.0)?;
    let ok = cfg.add_node(88.0, 44.0)?;
    let retryable = cfg.add_node(124.0, 48.0)?;
    let backoff = cfg.add_node(104.0, 44.0)?;
    let cleanup = cfg.add_node(104.0, 44.0)?;
    let error = cfg.add_node(88.0, 44.0)?;
    let exit = cfg.add_node(100.0, 44.0)?;

    cfg.add_edge(entry, acquire, EdgeKind::Default)?;
    cfg.add_edge(acquire, call, EdgeKind::Default)?;
    cfg.add_edge(call, ok, EdgeKind::Default)?;
    cfg.add_edge(ok, exit, EdgeKind::True)?;
    cfg.add_edge(ok, retryable, EdgeKind::False)?;
    cfg.add_edge(retryable, backoff, EdgeKind::True)?;
    cfg.add_edge(backoff, call, EdgeKind::Default)?;
    cfg.add_edge(retryable, cleanup, EdgeKind::False)?;
    cfg.add_edge(cleanup, error, EdgeKind::Default)?;

    Ok(cfg)
}

pub fn single_block_loop() -> Result<Cfg> {
    let mut cfg = Cfg::new();
    let entry = cfg.add_node(96.0, 44.0)?;
    let loop_body = cfg.add_node(148.0, 52.0)?;
    let exit = cfg.add_node(96.0, 44.0)?;

    cfg.add_edge(entry, loop_body, EdgeKind::Default)?;
    cfg.add_edge(loop_body, loop_body, EdgeKind::True)?;
    cfg.add_edge(loop_body, exit, EdgeKind::False)?;

    Ok(cfg)
}

pub fn all() -> Vec<(&'static str, Result<Cfg>)> {
    vec![
        ("branch_with_join", branch_with_join()),
        ("counted_loop", counted_loop()),
        ("nested_conditionals", nested_conditionals()),
        ("switch_dispatch", switch_dispatch()),
        ("retry_with_cleanup", retry_with_cleanup()),
        ("single_block_loop", single_block_loop()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_are_valid_and_nontrivial() {
        for (name, cfg) in all() {
            let cfg = cfg.unwrap_or_else(|error| panic!("{name} failed to build: {error}"));
            cfg.validate()
                .unwrap_or_else(|error| panic!("{name} failed validation: {error}"));
            assert!(
                cfg.block_count() >= 3,
                "{name} should have at least entry, body, and exit blocks"
            );
            assert!(
                cfg.edge_count() >= cfg.block_count(),
                "{name} should have nontrivial branching or looping"
            );
        }
    }

    #[test]
    fn examples_layout_with_triskel() {
        for (name, cfg) in all() {
            let layout = cfg
                .unwrap_or_else(|error| panic!("{name} failed to build: {error}"))
                .layout()
                .unwrap_or_else(|error| panic!("{name} failed layout: {error}"));
            assert!(!layout.blocks.is_empty(), "{name} should layout blocks");
            assert!(!layout.edges.is_empty(), "{name} should layout edges");
            assert!(layout.metrics().graph_area >= 0.0);
        }
    }
}
