#![no_std]

//! # Boarding Pass Smart Contract
//!
//! A digital boarding pass dApp for the travel domain built on Stellar / Soroban.
//!
//! ## Workflow
//!
//! 1. An **airline** calls `issue_pass` to mint a digital boarding pass for a
//!    passenger. The pass records the flight id, seat, gate and a 32-byte
//!    QR hash that the passenger will present at the gate.
//! 2. The **passenger** holds the pass and may call `cancel` to invalidate it
//!    before boarding (e.g. trip cancelled, rebooked, etc.).
//! 3. A **gate agent** calls `board` exactly once when scanning the QR hash at
//!    the gate. After boarding the pass cannot be used again — this prevents
//!    double-boarding and pass reuse fraud.
//! 4. Anyone (gate kiosk, lounge reader, etc.) may call `verify`, `get_seat`
//!    or `is_valid` to inspect a pass without changing its state.
//!
//! The contract does **not** move any XLM or other tokens. The "boarding pass"
//! is purely an on-chain data credential keyed by a 32-byte QR hash.

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

/// Lifecycle status of a boarding pass.
///
/// `Missing` (0) is never persisted — it is the implicit default returned by
/// `verify` when the pass id has not been issued. The stored values are
/// `Valid = 1`, `Boarded = 2`, `Cancelled = 3`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PassStatus {
    Missing,
    Valid,
    Boarded,
    Cancelled,
}

/// On-chain record for a single boarding pass.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardingPassData {
    /// Airline address that issued the pass.
    pub airline: Address,
    /// Passenger address that owns the pass.
    pub passenger: Address,
    /// Flight identifier, e.g. `"VN123"` or `"AA2026"`.
    pub flight_id: Symbol,
    /// Assigned seat number (1-based).
    pub seat: u32,
    /// Departure gate, e.g. `"A12"`.
    pub gate: Symbol,
    /// 32-byte QR code hash presented by the passenger at boarding.
    pub qr_hash: BytesN<32>,
    /// Current lifecycle status of the pass.
    pub status: PassStatus,
}

#[contract]
pub struct BoardingPass;

#[contractimpl]
impl BoardingPass {
    /// Issue a new digital boarding pass.
    ///
    /// The `airline` address authorizes the issuance. The `passenger` becomes
    /// the owner of the pass and the only one who may cancel it. The
    /// `qr_hash` is the unique 32-byte identifier that will be presented at
    /// the gate and is used as the storage key for the pass.
    ///
    /// # Panics
    /// * If the airline does not authorize the call.
    /// * If a pass with the same `qr_hash` already exists (no double issuance).
    pub fn issue_pass(
        env: Env,
        airline: Address,
        passenger: Address,
        flight_id: Symbol,
        seat: u32,
        gate: Symbol,
        qr_hash: BytesN<32>,
    ) {
        airline.require_auth();

        if env
            .storage()
            .instance()
            .get::<BytesN<32>, BoardingPassData>(&qr_hash)
            .is_some()
        {
            panic!("boarding pass already issued for this qr_hash");
        }

        let pass = BoardingPassData {
            airline,
            passenger,
            flight_id,
            seat,
            gate,
            qr_hash: qr_hash.clone(),
            status: PassStatus::Valid,
        };

        env.storage().instance().set(&qr_hash, &pass);
    }

    /// Mark a pass as boarded.
    ///
    /// Called by a `gate_agent` when the passenger scans their QR code at
    /// the gate. This is a one-time operation: once a pass is boarded it
    /// cannot be boarded or cancelled again, which prevents double-boarding
    /// and reuse of a stolen QR hash.
    ///
    /// # Panics
    /// * If the gate agent does not authorize the call.
    /// * If the pass has not been issued.
    /// * If the pass has already been boarded.
    /// * If the pass has been cancelled.
    pub fn board(env: Env, gate_agent: Address, pass_id: BytesN<32>) {
        gate_agent.require_auth();

        let mut pass: BoardingPassData = env
            .storage()
            .instance()
            .get(&pass_id)
            .unwrap_or_else(|| panic!("boarding pass not found"));

        match pass.status {
            PassStatus::Valid => {}
            PassStatus::Boarded => panic!("boarding pass already used"),
            PassStatus::Cancelled => panic!("boarding pass was cancelled"),
            PassStatus::Missing => panic!("boarding pass not found"),
        }

        pass.status = PassStatus::Boarded;
        env.storage().instance().set(&pass_id, &pass);
    }

    /// Cancel a boarding pass.
    ///
    /// Only the `passenger` named on the pass may cancel it. The cancellation
    /// reason is recorded as a contract event for downstream consumers
    /// (refund flows, airline dashboards, etc.). After cancellation the pass
    /// is permanently invalid and cannot be used to board.
    ///
    /// # Panics
    /// * If the passenger does not authorize the call.
    /// * If the pass has not been issued.
    /// * If the pass has already been boarded or cancelled.
    pub fn cancel(env: Env, passenger: Address, pass_id: BytesN<32>, reason: Symbol) {
        passenger.require_auth();

        let mut pass: BoardingPassData = env
            .storage()
            .instance()
            .get(&pass_id)
            .unwrap_or_else(|| panic!("boarding pass not found"));

        if pass.passenger != passenger {
            panic!("only the passenger may cancel this pass");
        }

        match pass.status {
            PassStatus::Valid => {}
            PassStatus::Boarded => panic!("cannot cancel a boarded pass"),
            PassStatus::Cancelled => panic!("boarding pass already cancelled"),
            PassStatus::Missing => panic!("boarding pass not found"),
        }

        pass.status = PassStatus::Cancelled;
        env.storage().instance().set(&pass_id, &pass);

        env.events()
            .publish((Symbol::new(&env, "cancelled"), pass_id), reason);
    }

    /// Return the numeric status of a pass.
    ///
    /// * `0` — pass not found (never issued)
    /// * `1` — `Valid` (issued, not yet boarded, not cancelled)
    /// * `2` — `Boarded` (already used at the gate)
    /// * `3` — `Cancelled` (cancelled by the passenger)
    pub fn verify(env: Env, pass_id: BytesN<32>) -> u32 {
        match env
            .storage()
            .instance()
            .get::<BytesN<32>, BoardingPassData>(&pass_id)
        {
            None => 0,
            Some(pass) => match pass.status {
                PassStatus::Missing => 0,
                PassStatus::Valid => 1,
                PassStatus::Boarded => 2,
                PassStatus::Cancelled => 3,
            },
        }
    }

    /// Return the seat number recorded on a pass.
    ///
    /// # Panics
    /// * If the pass has not been issued.
    pub fn get_seat(env: Env, pass_id: BytesN<32>) -> u32 {
        let pass: BoardingPassData = env
            .storage()
            .instance()
            .get(&pass_id)
            .unwrap_or_else(|| panic!("boarding pass not found"));
        pass.seat
    }

    /// Return `true` if the pass is issued and still in the `Valid` state —
    /// i.e. it has not been used to board and has not been cancelled.
    pub fn is_valid(env: Env, pass_id: BytesN<32>) -> bool {
        match env
            .storage()
            .instance()
            .get::<BytesN<32>, BoardingPassData>(&pass_id)
        {
            Some(pass) => pass.status == PassStatus::Valid,
            None => false,
        }
    }
}
