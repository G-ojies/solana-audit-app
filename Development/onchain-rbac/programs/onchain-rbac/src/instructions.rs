pub mod assign_role;
pub mod create_role;
pub mod initialize_organization;
pub mod perform_action;
pub mod revoke_membership;
pub mod set_membership_status;

// Glob re-exports are required by Anchor's `#[program]` macro, which resolves
// the generated `__client_accounts_*` / `__cpi_client_accounts_*` modules
// through `crate::*`. The only name collision is the per-module `handler`,
// which `lib.rs` always calls by fully-qualified path, so the warning is benign.
#[allow(ambiguous_glob_reexports)]
pub use assign_role::*;
#[allow(ambiguous_glob_reexports)]
pub use create_role::*;
#[allow(ambiguous_glob_reexports)]
pub use initialize_organization::*;
#[allow(ambiguous_glob_reexports)]
pub use perform_action::*;
#[allow(ambiguous_glob_reexports)]
pub use revoke_membership::*;
#[allow(ambiguous_glob_reexports)]
pub use set_membership_status::*;
