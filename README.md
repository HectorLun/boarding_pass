# boarding_pass

## Project Title
boarding_pass — On-chain digital boarding passes on Stellar / Soroban

## Project Description
Airline boarding passes are still mostly paper tickets, vendor-locked mobile apps or wallet PDFs that can be trivially screenshot, resold or reused at the gate. The `boarding_pass` dApp issues, cancels and validates a digital boarding pass as a small piece of on-chain state on Stellar. An airline mints a pass for a passenger; the passenger carries only a 32-byte QR hash; a gate agent calls `board` once to mark the passenger as boarded and the pass becomes permanently invalid, so the same QR can never be scanned twice.

## Project Vision
The long-term vision is a portable, airline-agnostic travel credential that any carrier, gate kiosk or loyalty program can read and verify without trusting a single proprietary app. By making the boarding pass a Soroban contract entry, the airline, the passenger and the gate agent each sign their own state transitions on-chain, giving travelers a verifiable, privacy-preserving record of their journey that survives app uninstalls, phone changes and airline bankruptcies.

## Key Features
- **Issue pass** — An authorized airline address mints a pass bound to a passenger address, a flight id, a seat, a gate and a unique 32-byte QR hash.
- **Cancel pass** — The named passenger can cancel a pass before boarding, with the reason emitted as a Soroban event for downstream refund / reporting flows.
- **One-time boarding** — A gate agent calls `board` to mark the pass as used; further calls on the same pass are rejected, preventing double-boarding and QR reuse.
- **Status verification** — A read-only `verify` function returns a numeric status code (`0` missing, `1` valid, `2` boarded, `3` cancelled) for any gate scanner.
- **Seat lookup** — A read-only `get_seat` function returns the assigned seat number for lounge displays and boarding announcements.
- **No XLM movement** — The contract is purely a data credential; no tokens are transferred, so the cost of issuing and validating a pass is dominated by Soroban fees only.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** travel dApp — see `contracts/boarding_pass/src/lib.rs` for the full boarding_pass business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** CDZF62OXXX2M2QFW6CBLYYZXFV4FVXKJKWIIY6TGEF3J3VHHJWW4CCR3
- **Explorer template:** https://stellar.expert/explorer/testnet/tx/186e4bc639ce39b43d9aff4242b3e1fc78a2edfff607ac790d26970a4d177346
![screenshot](https://i.ibb.co/Z62p2tP8/Screenshot-2026-06-05-221614.png)

## Future Scope
- **Reusable legs** — extend the contract so a single passenger address can hold many passes simultaneously (e.g. connecting flights) keyed by a `(passenger, qr_hash)` tuple.
- **Trustline-based refunds** — pair `cancel` with a token clawback / refund flow so cancelled passes can automatically trigger a partial refund of an airline-issued ticket token.
- **Off-chain QR payload** — keep the on-chain payload minimal (32-byte hash) while the richer boarding-pass metadata (boarding time, terminal, baggage tag) lives in IPFS and is referenced by a content hash on the pass.
- **Companion dApp** — build a small HTML / JS front-end that lets an airline issue, a passenger cancel and a gate agent scan & board directly from a Freighter-connected browser, exactly mirroring the dApp workflow in the bootcamp guide.
- **Multi-airline consortium mode** — let a governance address rotate which airline addresses are authorized to issue under a given flight id, so an alliance of carriers can share one boarding-pass contract.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `boarding_pass` (travel)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
