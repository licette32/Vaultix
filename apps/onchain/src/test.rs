use super::*;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_create_and_get_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 1u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 3000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Design"),
        },
        Milestone {
            amount: 3000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Dev"),
        },
        Milestone {
            amount: 4000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Deploy"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.depositor, depositor);
    assert_eq!(escrow.recipient, recipient);
    assert_eq!(escrow.total_amount, 10000);
    assert_eq!(escrow.total_released, 0);
    assert_eq!(escrow.status, EscrowStatus::Active);
    assert_eq!(escrow.milestones.len(), 3);

    assert_eq!(token_client.balance(&depositor), 0);
    assert_eq!(token_client.balance(&contract_id), 10000);
    assert_eq!(token_client.balance(&recipient), 0);
}

#[test]
fn test_buyer_confirm_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 2u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 5000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase1"),
        },
        Milestone {
            amount: 5000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase2"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &buyer,
        &seller,
        &milestones,
        &token_client.address,
    );

    client.confirm_delivery(&escrow_id, &0, &buyer);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.total_released, 5000);
    assert_eq!(
        escrow.milestones.get(0).unwrap().status,
        MilestoneStatus::Released
    );
    assert_eq!(
        escrow.milestones.get(1).unwrap().status,
        MilestoneStatus::Pending
    );

    assert_eq!(token_client.balance(&buyer), 0);
    assert_eq!(token_client.balance(&contract_id), 5000);
    assert_eq!(token_client.balance(&seller), 5000);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_dispute_blocks_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 9u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &1000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 500,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    client.raise_dispute(&escrow_id, &depositor);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);

    client.release_milestone(&escrow_id, &0);
}

#[test]
fn test_complete_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 3u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 5000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task1"),
        },
        Milestone {
            amount: 5000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task2"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    client.release_milestone(&escrow_id, &0);
    client.release_milestone(&escrow_id, &1);

    client.complete_escrow(&escrow_id);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Completed);
    assert_eq!(escrow.total_released, 10000);
}

#[test]
fn test_cancel_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 4u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 10000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Work"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    client.cancel_escrow(&escrow_id);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Cancelled);
}

#[test]
fn test_admin_resolves_dispute_to_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let escrow_id = 10u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    client.init(&admin);

    let milestones = vec![
        &env,
        Milestone {
            amount: 4000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase1"),
        },
        Milestone {
            amount: 6000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase2"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    client.raise_dispute(&escrow_id, &recipient);

    client.resolve_dispute(&escrow_id, &recipient);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Resolved);
    assert_eq!(escrow.resolution, Resolution::Recipient);
    assert_eq!(escrow.total_released, escrow.total_amount);
    assert!(escrow
        .milestones
        .iter()
        .all(|m| m.status == MilestoneStatus::Released));

    assert_eq!(token_client.balance(&recipient), 10000);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(token_client.balance(&depositor), 0);
}

#[test]
fn test_admin_resolves_dispute_to_depositor() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let escrow_id = 11u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &5000);

    client.init(&admin);

    let milestones = vec![
        &env,
        Milestone {
            amount: 2000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Alpha"),
        },
        Milestone {
            amount: 3000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Beta"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    client.raise_dispute(&escrow_id, &depositor);

    client.resolve_dispute(&escrow_id, &depositor);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Resolved);
    assert_eq!(escrow.resolution, Resolution::Depositor);
    assert_eq!(escrow.total_released, 0);
    assert!(escrow
        .milestones
        .iter()
        .all(|m| m.status == MilestoneStatus::Disputed));

    assert_eq!(token_client.balance(&depositor), 5000);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(token_client.balance(&recipient), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_duplicate_escrow_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 5u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 1000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Test"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );
    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_double_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 6u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 1000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );
    client.release_milestone(&escrow_id, &0);
    client.release_milestone(&escrow_id, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_too_many_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 7u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let mut milestones = Vec::new(&env);
    for _i in 0..21 {
        milestones.push_back(Milestone {
            amount: 100,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        });
    }

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_invalid_milestone_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 8u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 0, // Invalid: zero amount
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_unauthorized_confirm_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let non_buyer = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 9u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 1000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &buyer,
        &seller,
        &milestones,
        &token_client.address,
    );

    client.confirm_delivery(&escrow_id, &0, &non_buyer);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_double_confirm_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 10u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 1000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    client.create_escrow(
        &escrow_id,
        &buyer,
        &seller,
        &milestones,
        &token_client.address,
    );

    client.confirm_delivery(&escrow_id, &0, &buyer);

    client.confirm_delivery(&escrow_id, &0, &buyer);
}

#[test]
fn test_zero_amount_milestone_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 11u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 0, // Invalid: zero amount
            status: MilestoneStatus::Pending,
            description: symbol_short!("Test"),
        },
    ];

    let result = client.try_create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    assert_eq!(result, Err(Ok(Error::ZeroAmount)));
}

#[test]
fn test_negative_amount_milestone_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 12u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: -1000, // Invalid: negative amount
            status: MilestoneStatus::Pending,
            description: symbol_short!("Test"),
        },
    ];

    let result = client.try_create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    assert_eq!(result, Err(Ok(Error::ZeroAmount)));
}

#[test]
fn test_self_dealing_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let same_party = Address::generate(&env); // Same address for both
    let admin = Address::generate(&env);
    let escrow_id = 13u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&same_party, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 5000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Task"),
        },
    ];

    let result = client.try_create_escrow(
        &escrow_id,
        &same_party,
        &same_party,
        &milestones,
        &token_client.address,
    );

    assert_eq!(result, Err(Ok(Error::SelfDealing)));
}

#[test]
fn test_valid_escrow_creation_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultixEscrow, ());
    let client = VaultixEscrowClient::new(&env, &contract_id);

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_id = 14u64;

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&depositor, &10000);

    let milestones = vec![
        &env,
        Milestone {
            amount: 3000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase1"),
        },
        Milestone {
            amount: 7000,
            status: MilestoneStatus::Pending,
            description: symbol_short!("Phase2"),
        },
    ];

    let result = client.try_create_escrow(
        &escrow_id,
        &depositor,
        &recipient,
        &milestones,
        &token_client.address,
    );

    assert!(result.is_ok());

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.depositor, depositor);
    assert_eq!(escrow.recipient, recipient);
    assert_eq!(escrow.total_amount, 10000);
}
