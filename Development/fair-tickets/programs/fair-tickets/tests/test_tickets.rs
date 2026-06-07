use {
    anchor_lang::{
        solana_program::instruction::Instruction, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn system_program() -> Pubkey {
    Pubkey::default()
}

const PRICE: u64 = 100_000_000; // 0.1 SOL
const CAP: u64 = 120_000_000; // 0.12 SOL resale ceiling

fn setup() -> (LiteSVM, Pubkey) {
    let program_id = fair_tickets::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/fair_tickets.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    (svm, program_id)
}

fn event_pda(organizer: &Pubkey, event_id: u64, pid: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[fair_tickets::EVENT_SEED, organizer.as_ref(), &event_id.to_le_bytes()],
        pid,
    )
    .0
}

fn ticket_pda(event: &Pubkey, serial: u32, pid: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[fair_tickets::TICKET_SEED, event.as_ref(), &serial.to_le_bytes()],
        pid,
    )
    .0
}

fn send(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    data: Vec<u8>,
    metas: Vec<anchor_lang::solana_program::instruction::AccountMeta>,
    payer: &Keypair,
) -> Result<(), String> {
    let ix = Instruction { program_id, accounts: metas, data };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer])
        .map_err(|e| e.to_string())?;
    svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{:?}", e.err))
}

fn fund(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    kp
}

fn read_ticket(svm: &LiteSVM, pda: &Pubkey) -> fair_tickets::Ticket {
    let acct = svm.get_account(pda).expect("ticket account");
    fair_tickets::Ticket::try_deserialize(&mut &acct.data[..]).unwrap()
}

fn read_event(svm: &LiteSVM, pda: &Pubkey) -> fair_tickets::Event {
    let acct = svm.get_account(pda).expect("event account");
    fair_tickets::Event::try_deserialize(&mut &acct.data[..]).unwrap()
}

fn create_event(svm: &mut LiteSVM, pid: Pubkey, organizer: &Keypair, eid: u64) -> Pubkey {
    let evt = event_pda(&organizer.pubkey(), eid, &pid);
    send(
        svm,
        pid,
        fair_tickets::instruction::CreateEvent {
            event_id: eid,
            name: "Concert".to_string(),
            price: PRICE,
            max_resale_price: CAP,
            supply: 5,
        }
        .data(),
        fair_tickets::accounts::CreateEvent {
            event: evt,
            organizer: organizer.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        organizer,
    )
    .expect("create_event");
    evt
}

fn buy_ticket(svm: &mut LiteSVM, pid: Pubkey, evt: Pubkey, organizer: &Pubkey,
              buyer: &Keypair, serial: u32) -> Pubkey {
    let tkt = ticket_pda(&evt, serial, &pid);
    send(
        svm,
        pid,
        fair_tickets::instruction::BuyTicket {}.data(),
        fair_tickets::accounts::BuyTicket {
            event: evt,
            ticket: tkt,
            buyer: buyer.pubkey(),
            organizer: *organizer,
            system_program: system_program(),
        }
        .to_account_metas(None),
        buyer,
    )
    .expect("buy_ticket");
    tkt
}

fn list_resale(svm: &mut LiteSVM, pid: Pubkey, evt: Pubkey, tkt: Pubkey,
               owner: &Keypair, price: u64) -> Result<(), String> {
    send(
        svm,
        pid,
        fair_tickets::instruction::ListResale { price }.data(),
        fair_tickets::accounts::ListResale { event: evt, ticket: tkt, owner: owner.pubkey() }
            .to_account_metas(None),
        owner,
    )
}

#[test]
fn primary_sale_works() {
    let (mut svm, pid) = setup();
    let organizer = fund(&mut svm);
    let buyer = fund(&mut svm);
    let evt = create_event(&mut svm, pid, &organizer, 1);

    let org_before = svm.get_balance(&organizer.pubkey()).unwrap();
    let tkt = buy_ticket(&mut svm, pid, evt, &organizer.pubkey(), &buyer, 0);
    let org_after = svm.get_balance(&organizer.pubkey()).unwrap();

    assert_eq!(read_event(&svm, &evt).sold, 1);
    let t = read_ticket(&svm, &tkt);
    assert_eq!(t.owner, buyer.pubkey());
    assert_eq!(t.paid, PRICE);
    assert_eq!(org_after - org_before, PRICE, "organizer should receive face price");
}

#[test]
fn resale_above_cap_rejected() {
    let (mut svm, pid) = setup();
    let organizer = fund(&mut svm);
    let buyer = fund(&mut svm);
    let evt = create_event(&mut svm, pid, &organizer, 1);
    let tkt = buy_ticket(&mut svm, pid, evt, &organizer.pubkey(), &buyer, 0);

    // Above the cap → rejected (anti-scalp guarantee).
    assert!(
        list_resale(&mut svm, pid, evt, tkt, &buyer, CAP + 1).is_err(),
        "listing above cap should fail"
    );
    // At the cap → allowed.
    list_resale(&mut svm, pid, evt, tkt, &buyer, CAP).expect("listing at cap should pass");
    let t = read_ticket(&svm, &tkt);
    assert!(t.for_sale && t.resale_price == CAP);
}

#[test]
fn buy_resale_transfers_ownership_and_funds() {
    let (mut svm, pid) = setup();
    let organizer = fund(&mut svm);
    let seller = fund(&mut svm);
    let buyer2 = fund(&mut svm);
    let evt = create_event(&mut svm, pid, &organizer, 1);
    let tkt = buy_ticket(&mut svm, pid, evt, &organizer.pubkey(), &seller, 0);

    let resale = 110_000_000u64; // <= CAP
    list_resale(&mut svm, pid, evt, tkt, &seller, resale).expect("list");

    let seller_before = svm.get_balance(&seller.pubkey()).unwrap();
    send(
        &mut svm,
        pid,
        fair_tickets::instruction::BuyResale {}.data(),
        fair_tickets::accounts::BuyResale {
            event: evt,
            ticket: tkt,
            buyer: buyer2.pubkey(),
            seller: seller.pubkey(),
            system_program: system_program(),
        }
        .to_account_metas(None),
        &buyer2,
    )
    .expect("buy_resale");
    let seller_after = svm.get_balance(&seller.pubkey()).unwrap();

    let t = read_ticket(&svm, &tkt);
    assert_eq!(t.owner, buyer2.pubkey(), "ownership should transfer");
    assert!(!t.for_sale);
    assert_eq!(seller_after - seller_before, resale, "seller receives resale price");
}

#[test]
fn redeemed_ticket_cannot_be_resold() {
    let (mut svm, pid) = setup();
    let organizer = fund(&mut svm);
    let buyer = fund(&mut svm);
    let evt = create_event(&mut svm, pid, &organizer, 1);
    let tkt = buy_ticket(&mut svm, pid, evt, &organizer.pubkey(), &buyer, 0);

    // Organizer checks the ticket in.
    send(
        &mut svm,
        pid,
        fair_tickets::instruction::Redeem {}.data(),
        fair_tickets::accounts::Redeem { event: evt, ticket: tkt, organizer: organizer.pubkey() }
            .to_account_metas(None),
        &organizer,
    )
    .expect("redeem");
    assert!(read_ticket(&svm, &tkt).redeemed);

    // Listing a redeemed ticket must fail.
    assert!(
        list_resale(&mut svm, pid, evt, tkt, &buyer, CAP).is_err(),
        "redeemed ticket should not be listable"
    );
}

#[test]
fn non_owner_cannot_list() {
    let (mut svm, pid) = setup();
    let organizer = fund(&mut svm);
    let buyer = fund(&mut svm);
    let imposter = fund(&mut svm);
    let evt = create_event(&mut svm, pid, &organizer, 1);
    let tkt = buy_ticket(&mut svm, pid, evt, &organizer.pubkey(), &buyer, 0);

    assert!(
        list_resale(&mut svm, pid, evt, tkt, &imposter, CAP).is_err(),
        "non-owner should not be able to list"
    );
}
