use {
    anchor_lang::{
        solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

// The System Program id is 32 zero bytes (base58 "111...111").
fn system_program() -> Pubkey {
    Pubkey::default()
}

// Permission bits (mirror constants.rs).
const PERM_READ: u64 = 1 << 0;
const PERM_WRITE: u64 = 1 << 1;

fn setup() -> (LiteSVM, Pubkey) {
    let program_id = onchain_rbac::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/onchain_rbac.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    (svm, program_id)
}

fn org_pda(admin: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[onchain_rbac::ORG_SEED, admin.as_ref()], program_id).0
}

fn role_pda(org: &Pubkey, id: u32, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[onchain_rbac::ROLE_SEED, org.as_ref(), &id.to_le_bytes()],
        program_id,
    )
    .0
}

fn member_pda(org: &Pubkey, user: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[onchain_rbac::MEMBER_SEED, org.as_ref(), user.as_ref()],
        program_id,
    )
    .0
}

fn send(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    data: Vec<u8>,
    metas: Vec<anchor_lang::solana_program::instruction::AccountMeta>,
    payer: &Keypair,
    extra_signers: &[&Keypair],
) -> Result<(), String> {
    let ix = Instruction {
        program_id,
        accounts: metas,
        data,
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers[..])
        .map_err(|e| e.to_string())?;
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e.err))
}

/// Helper: stand up an org with an admin role (read+write) and a viewer role
/// (read only), returning (svm, program_id, admin, org).
fn bootstrap_org() -> (LiteSVM, Pubkey, Keypair, Pubkey) {
    let (mut svm, program_id) = setup();
    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    let org = org_pda(&admin.pubkey(), &program_id);

    // initialize_organization
    send(
        &mut svm,
        program_id,
        onchain_rbac::instruction::InitializeOrganization {}.data(),
        onchain_rbac::accounts::InitializeOrganization {
            organization: org,
            admin: admin.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        &admin,
        &[],
    )
    .expect("init org");

    // create_role: editor (id 0) = read | write
    send(
        &mut svm,
        program_id,
        onchain_rbac::instruction::CreateRole {
            id: 0,
            name: "editor".to_string(),
            permissions: PERM_READ | PERM_WRITE,
        }
        .data(),
        onchain_rbac::accounts::CreateRole {
            organization: org,
            role: role_pda(&org, 0, &program_id),
            admin: admin.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        &admin,
        &[],
    )
    .expect("create editor role");

    // create_role: viewer (id 1) = read only
    send(
        &mut svm,
        program_id,
        onchain_rbac::instruction::CreateRole {
            id: 1,
            name: "viewer".to_string(),
            permissions: PERM_READ,
        }
        .data(),
        onchain_rbac::accounts::CreateRole {
            organization: org,
            role: role_pda(&org, 1, &program_id),
            admin: admin.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        &admin,
        &[],
    )
    .expect("create viewer role");

    (svm, program_id, admin, org)
}

fn assign(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    admin: &Keypair,
    org: Pubkey,
    role_id: u32,
    user: &Pubkey,
) -> Result<(), String> {
    send(
        svm,
        program_id,
        onchain_rbac::instruction::AssignRole { user: *user }.data(),
        onchain_rbac::accounts::AssignRole {
            organization: org,
            role: role_pda(&org, role_id, &program_id),
            membership: member_pda(&org, user, &program_id),
            admin: admin.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        admin,
        &[],
    )
}

fn perform(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    org: Pubkey,
    role_id: u32,
    user: &Keypair,
    required: u64,
) -> Result<(), String> {
    send(
        svm,
        program_id,
        onchain_rbac::instruction::PerformAction {
            required_permission: required,
        }
        .data(),
        onchain_rbac::accounts::PerformAction {
            organization: org,
            role: role_pda(&org, role_id, &program_id),
            membership: member_pda(&org, &user.pubkey(), &program_id),
            user: user.pubkey(),
        }
        .to_account_metas(None),
        user,
        &[],
    )
}

#[test]
fn editor_can_write() {
    let (mut svm, program_id, admin, org) = bootstrap_org();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    assign(&mut svm, program_id, &admin, org, 0, &user.pubkey()).expect("assign editor");
    perform(&mut svm, program_id, org, 0, &user, PERM_WRITE).expect("editor performs write");
}

#[test]
fn viewer_cannot_write() {
    let (mut svm, program_id, admin, org) = bootstrap_org();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    assign(&mut svm, program_id, &admin, org, 1, &user.pubkey()).expect("assign viewer");
    // viewer holds read only — a write action must be denied
    let res = perform(&mut svm, program_id, org, 1, &user, PERM_WRITE);
    assert!(res.is_err(), "viewer should be denied write, got Ok");
    // ...but a read action is allowed
    perform(&mut svm, program_id, org, 1, &user, PERM_READ).expect("viewer performs read");
}

#[test]
fn deactivated_membership_is_denied() {
    let (mut svm, program_id, admin, org) = bootstrap_org();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    assign(&mut svm, program_id, &admin, org, 0, &user.pubkey()).expect("assign editor");

    // admin deactivates the membership
    send(
        &mut svm,
        program_id,
        onchain_rbac::instruction::SetMembershipStatus { active: false }.data(),
        onchain_rbac::accounts::SetMembershipStatus {
            organization: org,
            membership: member_pda(&org, &user.pubkey(), &program_id),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        &admin,
        &[],
    )
    .expect("deactivate membership");

    let res = perform(&mut svm, program_id, org, 0, &user, PERM_READ);
    assert!(res.is_err(), "inactive membership should be denied, got Ok");
}

#[test]
fn non_admin_cannot_create_role() {
    let (mut svm, program_id, _admin, org) = bootstrap_org();
    let imposter = Keypair::new();
    svm.airdrop(&imposter.pubkey(), 1_000_000_000).unwrap();

    // imposter is not the org admin; has_one = admin must reject this.
    let res = send(
        &mut svm,
        program_id,
        onchain_rbac::instruction::CreateRole {
            id: 99,
            name: "rogue".to_string(),
            permissions: u64::MAX,
        }
        .data(),
        onchain_rbac::accounts::CreateRole {
            organization: org,
            role: role_pda(&org, 99, &program_id),
            admin: imposter.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        &imposter,
        &[],
    );
    assert!(res.is_err(), "non-admin should not create roles, got Ok");
}
