use anchor_lang::prelude::*;

#[error_code]
pub enum RbacError {
    #[msg("Caller's role does not grant the required permission")]
    PermissionDenied,
    #[msg("Membership is inactive")]
    MembershipInactive,
    #[msg("Role does not belong to the provided organization")]
    RoleMismatch,
    #[msg("Role name exceeds the maximum length")]
    NameTooLong,
}
