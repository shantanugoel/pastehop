use crate::{errors::PasteHopError, target::ResolvedTarget, transport::Transport};

pub fn run_gc(
    transport: &Transport,
    target: &ResolvedTarget,
    ttl_hours: u64,
    dry_run: bool,
) -> Result<Vec<String>, PasteHopError> {
    transport.gc_expired(&target.host, &target.remote_dir, ttl_hours, dry_run)
}
