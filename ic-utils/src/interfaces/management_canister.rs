use crate::call::AsyncCall;
use crate::canister::CanisterBuilder;
use crate::Canister;
use candid::{CandidType, Deserialize};
use ic_agent::export::Principal;
use ic_agent::Agent;
use std::fmt::Debug;

pub mod attributes;
pub mod builders;
pub use builders::{CreateCanisterBuilder, InstallCodeBuilder};
use std::convert::From;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct ManagementCanister;

impl ManagementCanister {
    /// Create an instance of a [Canister] implementing the ManagementCanister interface
    /// and pointing to the right Canister ID.
    pub fn create(agent: &Agent) -> Canister<ManagementCanister> {
        Canister::builder()
            .with_agent(agent)
            .with_canister_id(Principal::management_canister())
            .with_interface(ManagementCanister)
            .build()
            .unwrap()
    }

    /// Creating a CanisterBuilder with the right interface and Canister Id. This can
    /// be useful, for example, for providing additional Builder information.
    pub fn with_agent(agent: &Agent) -> CanisterBuilder<ManagementCanister> {
        Canister::builder()
            .with_agent(agent)
            .with_canister_id(Principal::management_canister())
            .with_interface(ManagementCanister)
    }
}

/// The complete canister status information of a canister. This includes
/// the CanisterStatus, a hash of the module installed on the canister (None if nothing installed),
/// the contoller of the canister, the canisters memory size, and its balance in cycles.
#[derive(Clone, Debug, Deserialize)]
pub struct StatusCallResult {
    pub status: CanisterStatus,
    pub module_hash: Option<Vec<u8>>,
    pub controller: Principal,
    pub memory_size: candid::Nat,
    pub cycles: candid::Nat,
}

impl std::fmt::Display for StatusCallResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// The status of a Canister, whether it's running, in the process of stopping, or
/// stopped.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CanisterStatus {
    Running,
    Stopping,
    Stopped,
}

impl std::fmt::Display for CanisterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<'agent> Canister<'agent, ManagementCanister> {
    /// Get the status of a canister.
    pub fn canister_status<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
    ) -> impl 'agent + AsyncCall<(StatusCallResult,)> {
        #[derive(CandidType)]
        struct In {
            canister_id: Principal,
        }

        self.update_("canister_status")
            .with_arg(In {
                canister_id: canister_id.clone(),
            })
            .build()
            .map(|result: (StatusCallResult,)| (result.0,))
    }

    /// Create a canister.
    pub fn create_canister<'canister: 'agent>(
        &'canister self,
    ) -> CreateCanisterBuilder<'agent, 'canister, ManagementCanister> {
        CreateCanisterBuilder::builder(self)
    }

    /// This method deposits the cycles included in this call into the specified canister.
    /// Only the controller of the canister can deposit cycles.
    pub fn deposit_cycles<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
    ) -> impl 'agent + AsyncCall<()> {
        #[derive(CandidType)]
        struct Argument {
            canister_id: Principal,
        }

        self.update_("deposit_cycles")
            .with_arg(Argument {
                canister_id: canister_id.clone(),
            })
            .build()
    }

    /// Deletes a canister.
    pub fn delete_canister<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
    ) -> impl 'agent + AsyncCall<()> {
        #[derive(CandidType)]
        struct Argument {
            canister_id: Principal,
        }

        self.update_("delete_canister")
            .with_arg(Argument {
                canister_id: canister_id.clone(),
            })
            .build()
    }

    /// Until developers can convert real ICP tokens to provision a new canister with cycles,
    /// the system provides the provisional_create_canister_with_cycles method.
    /// It behaves as create_canister, but initializes the canister’s balance with amount fresh cycles
    /// (using MAX_CANISTER_BALANCE if amount = null, else capping the balance at MAX_CANISTER_BALANCE).
    /// Cycles added to this call via ic0.call_cycles_add are returned to the caller.
    /// This method is only available in local development instances, and will be removed in the future.
    pub fn provisional_create_canister_with_cycles<'canister: 'agent>(
        &'canister self,
        amount: Option<u64>,
    ) -> impl 'agent + AsyncCall<(Principal,)> {
        #[derive(CandidType)]
        struct Argument {
            amount: Option<candid::Nat>,
        }

        #[derive(Deserialize)]
        struct Out {
            canister_id: Principal,
        }

        self.update_("provisional_create_canister_with_cycles")
            .with_arg(Argument {
                amount: amount.map(candid::Nat::from),
            })
            .build()
            .map(|result: (Out,)| (result.0.canister_id,))
    }

    /// Until developers can convert real ICP tokens to a top up an existing canister,
    /// the system provides the provisional_top_up_canister method.
    /// It adds amount cycles to the balance of canister identified by amount
    /// (implicitly capping it at MAX_CANISTER_BALANCE).
    pub fn provisional_top_up_canister<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
        amount: u64,
    ) -> impl 'agent + AsyncCall<()> {
        #[derive(CandidType)]
        struct Argument {
            canister_id: Principal,
            amount: u64,
        }

        self.update_("provisional_top_up_canister")
            .with_arg(Argument {
                canister_id: canister_id.clone(),
                amount,
            })
            .build()
    }

    /// This method takes no input and returns 32 pseudo-random bytes to the caller.
    /// The return value is unknown to any part of the IC at time of the submission of this call.
    /// A new return value is generated for each call to this method.
    pub fn raw_rand<'canister: 'agent>(&'canister self) -> impl 'agent + AsyncCall<(Vec<u8>,)> {
        self.update_("raw_rand")
            .build()
            .map(|result: (Vec<u8>,)| (result.0,))
    }

    /// Starts a canister.
    pub fn start_canister<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
    ) -> impl 'agent + AsyncCall<()> {
        #[derive(CandidType)]
        struct Argument {
            canister_id: Principal,
        }

        self.update_("start_canister")
            .with_arg(Argument {
                canister_id: canister_id.clone(),
            })
            .build()
    }

    /// Stop a canister.
    pub fn stop_canister<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
    ) -> impl 'agent + AsyncCall<()> {
        #[derive(CandidType)]
        struct Argument {
            canister_id: Principal,
        }

        self.update_("stop_canister")
            .with_arg(Argument {
                canister_id: canister_id.clone(),
            })
            .build()
    }

    /// Install a canister, with all the arguments necessary for creating the canister.
    pub fn install_code<'canister: 'agent>(
        &'canister self,
        canister_id: &Principal,
        wasm: &'canister [u8],
    ) -> InstallCodeBuilder<'agent, 'canister, ManagementCanister> {
        InstallCodeBuilder::builder(self, canister_id, wasm)
    }
}
