#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
    Vec,
};

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MilestoneStatus {
    Pending,
    Released,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Milestone {
    pub amount: i128,
    pub status: MilestoneStatus,
    pub description: Symbol,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscrowStatus {
    Active,
    Completed,
    Cancelled,
    Disputed,
    Resolved,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resolution {
    None,
    Depositor,
    Recipient,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Escrow {
    pub depositor: Address,
    pub recipient: Address,
    pub total_amount: i128,
    pub total_released: i128,
    pub milestones: Vec<Milestone>,
    pub token: Address,
    pub status: EscrowStatus,
    pub resolution: Resolution,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EscrowNotFound = 1,
    EscrowAlreadyExists = 2,
    MilestoneNotFound = 3,
    MilestoneAlreadyReleased = 4,
    UnauthorizedAccess = 5,
    InvalidMilestoneAmount = 6,
    TotalAmountMismatch = 7,
    InsufficientBalance = 8,
    EscrowNotActive = 9,
    VectorTooLarge = 10,
    ZeroAmount = 11,
    InvalidDeadline = 12,
    SelfDealing = 13,
    AdminNotInitialized = 14,
    AlreadyInitialized = 15,
    InvalidEscrowStatus = 16,
    AlreadyInDispute = 17,
    InvalidWinner = 18,
}

#[contract]
pub struct VaultixEscrow;

#[contractimpl]
impl VaultixEscrow {
    /// Initializes the contract with an admin address responsible for dispute resolution.
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().persistent().has(&admin_storage_key()) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().persistent().set(&admin_storage_key(), &admin);
        Ok(())
    }

    /// Creates a new escrow with milestone-based payment releases.
    pub fn create_escrow(
        env: Env,
        escrow_id: u64,
        depositor: Address,
        recipient: Address,
        milestones: Vec<Milestone>,
        token: Address,
    ) -> Result<(), Error> {
        depositor.require_auth();

        if depositor == recipient {
            return Err(Error::SelfDealing);
        }

        let storage_key = get_storage_key(escrow_id);
        if env.storage().persistent().has(&storage_key) {
            return Err(Error::EscrowAlreadyExists);
        }

        let total_amount = validate_milestones(&milestones)?;

        let mut initialized_milestones = Vec::new(&env);
        for milestone in milestones.iter() {
            let mut m = milestone.clone();
            m.status = MilestoneStatus::Pending;
            initialized_milestones.push_back(m);
        }

        let escrow = Escrow {
            depositor: depositor.clone(),
            recipient,
            total_amount,
            total_released: 0,
            milestones: initialized_milestones,
            token: token.clone(),
            status: EscrowStatus::Active,
            resolution: Resolution::None,
        };

        env.storage().persistent().set(&storage_key, &escrow);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, env.current_contract_address(), &total_amount);

        Ok(())
    }

    /// Retrieves escrow details (read-only)
    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, Error> {
        let storage_key = get_storage_key(escrow_id);
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)
    }

    /// Read-only helper to fetch escrow status
    pub fn get_state(env: Env, escrow_id: u64) -> Result<EscrowStatus, Error> {
        let escrow = Self::get_escrow(env, escrow_id)?;
        Ok(escrow.status)
    }

    /// Releases a specific milestone payment to the recipient (depositor-driven path).
    pub fn release_milestone(env: Env, escrow_id: u64, milestone_index: u32) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        escrow.depositor.require_auth();

        if escrow.status != EscrowStatus::Active {
            return Err(Error::EscrowNotActive);
        }

        if milestone_index >= escrow.milestones.len() {
            return Err(Error::MilestoneNotFound);
        }

        let mut milestone = escrow
            .milestones
            .get(milestone_index)
            .ok_or(Error::MilestoneNotFound)?;

        if milestone.status == MilestoneStatus::Released {
            return Err(Error::MilestoneAlreadyReleased);
        }

        milestone.status = MilestoneStatus::Released;
        escrow.milestones.set(milestone_index, milestone.clone());

        escrow.total_released = escrow
            .total_released
            .checked_add(milestone.amount)
            .ok_or(Error::InvalidMilestoneAmount)?;

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.recipient,
            &milestone.amount,
        );

        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }

    /// Buyer confirms delivery and releases a milestone to the recipient (buyer-driven path).
    pub fn confirm_delivery(
        env: Env,
        escrow_id: u64,
        milestone_index: u32,
        buyer: Address,
    ) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        buyer.require_auth();

        if escrow.depositor != buyer {
            return Err(Error::UnauthorizedAccess);
        }

        if escrow.status != EscrowStatus::Active {
            return Err(Error::EscrowNotActive);
        }

        if milestone_index >= escrow.milestones.len() {
            return Err(Error::MilestoneNotFound);
        }

        let mut milestone = escrow
            .milestones
            .get(milestone_index)
            .ok_or(Error::MilestoneNotFound)?;

        if milestone.status == MilestoneStatus::Released {
            return Err(Error::MilestoneAlreadyReleased);
        }

        milestone.status = MilestoneStatus::Released;
        escrow.milestones.set(milestone_index, milestone.clone());

        escrow.total_released = escrow
            .total_released
            .checked_add(milestone.amount)
            .ok_or(Error::InvalidMilestoneAmount)?;

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.recipient,
            &milestone.amount,
        );

        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }

    /// Raises a dispute on an active escrow. Either party (depositor or recipient) may invoke this.
    pub fn raise_dispute(env: Env, escrow_id: u64, caller: Address) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        if caller != escrow.depositor && caller != escrow.recipient {
            return Err(Error::UnauthorizedAccess);
        }
        caller.require_auth();

        if escrow.status == EscrowStatus::Disputed {
            return Err(Error::AlreadyInDispute);
        }
        if escrow.status != EscrowStatus::Active {
            return Err(Error::InvalidEscrowStatus);
        }

        let mut updated_milestones = Vec::new(&env);
        for milestone in escrow.milestones.iter() {
            let mut m = milestone.clone();
            if m.status == MilestoneStatus::Pending {
                m.status = MilestoneStatus::Disputed;
            }
            updated_milestones.push_back(m);
        }

        escrow.milestones = updated_milestones;
        escrow.status = EscrowStatus::Disputed;
        escrow.resolution = Resolution::None;
        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }

    /// Resolves an active dispute by directing funds to the chosen party. Only the admin may call this.
    pub fn resolve_dispute(env: Env, escrow_id: u64, winner: Address) -> Result<(), Error> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(Error::InvalidEscrowStatus);
        }

        if winner != escrow.depositor && winner != escrow.recipient {
            return Err(Error::InvalidWinner);
        }

        let outstanding = escrow
            .total_amount
            .checked_sub(escrow.total_released)
            .ok_or(Error::InvalidMilestoneAmount)?;

        let token_client = token::Client::new(&env, &escrow.token);

        if winner == escrow.recipient {
            let mut updated_milestones = Vec::new(&env);
            for milestone in escrow.milestones.iter() {
                let mut m = milestone.clone();
                if m.status != MilestoneStatus::Released {
                    m.status = MilestoneStatus::Released;
                }
                updated_milestones.push_back(m);
            }
            escrow.milestones = updated_milestones;
            escrow.total_released = escrow.total_amount;
            escrow.resolution = Resolution::Recipient;

            if outstanding > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &escrow.recipient,
                    &outstanding,
                );
            }
        } else {
            let mut updated_milestones = Vec::new(&env);
            for milestone in escrow.milestones.iter() {
                let mut m = milestone.clone();
                if m.status == MilestoneStatus::Pending || m.status == MilestoneStatus::Disputed {
                    m.status = MilestoneStatus::Disputed;
                }
                updated_milestones.push_back(m);
            }
            escrow.milestones = updated_milestones;
            escrow.resolution = Resolution::Depositor;

            if outstanding > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &escrow.depositor,
                    &outstanding,
                );
            }
        }

        escrow.status = EscrowStatus::Resolved;
        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }

    /// Cancels an escrow before any milestones are released.
    pub fn cancel_escrow(env: Env, escrow_id: u64) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        escrow.depositor.require_auth();

        if escrow.status != EscrowStatus::Active {
            return Err(Error::InvalidEscrowStatus);
        }

        if escrow.total_released > 0 {
            return Err(Error::MilestoneAlreadyReleased);
        }

        escrow.status = EscrowStatus::Cancelled;
        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }

    /// Marks an escrow as completed after all milestones are released.
    pub fn complete_escrow(env: Env, escrow_id: u64) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        escrow.depositor.require_auth();

        if escrow.status != EscrowStatus::Active {
            return Err(Error::InvalidEscrowStatus);
        }

        if !verify_all_released(&escrow.milestones) {
            return Err(Error::EscrowNotActive);
        }

        escrow.status = EscrowStatus::Completed;
        env.storage().persistent().set(&storage_key, &escrow);

        Ok(())
    }
}

fn get_storage_key(escrow_id: u64) -> (Symbol, u64) {
    (symbol_short!("escrow"), escrow_id)
}

fn admin_storage_key() -> Symbol {
    symbol_short!("admin")
}

fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .persistent()
        .get(&admin_storage_key())
        .ok_or(Error::AdminNotInitialized)
}

fn validate_milestones(milestones: &Vec<Milestone>) -> Result<i128, Error> {
    if milestones.len() > 20 {
        return Err(Error::VectorTooLarge);
    }

    let mut total: i128 = 0;

    for milestone in milestones.iter() {
        if milestone.amount <= 0 {
            return Err(Error::ZeroAmount);
        }

        total = total
            .checked_add(milestone.amount)
            .ok_or(Error::InvalidMilestoneAmount)?;
    }

    Ok(total)
}

fn verify_all_released(milestones: &Vec<Milestone>) -> bool {
    for milestone in milestones.iter() {
        if milestone.status != MilestoneStatus::Released {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test;
