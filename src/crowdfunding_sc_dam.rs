
#![no_std]

use multiversx_sc::derive_imports::*;
#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Clone, Copy)]
pub enum Status {
    FundingPeriod,
    Successful,
    Failed,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait CrowdfundingScDAM {
    #[init]
    fn init(&self, target_max: BigUint,target_min: BigUint, deadline: u64) {
        require!(target_max > 0 && target_min > 0, "L'objectiu ha de ser superior a 0 EGLD");
        require!(target_max > target_min , "L'objectiu màxim ha de ser superior a l'objectiu mínim");
        self.target_max().set(target_max);
        self.target_min().set(target_min);

        require!(
            deadline > self.get_current_time(),
            "La data límit no pot estar al passat"
        );
        self.deadline().set(deadline);
    }

    /*
    #[upgrade]
    fn upgrade_target(&self, new_cap: BigUint) {
        require!(new_cap > self.total_raised().get(), 
        "El nou objectiu ha de ser superior a la quantitat acumulada actual!"
        );
        self.target().set(&new_cap);
    }*/

    #[upgrade]
    fn upgrade_target(&self) {}

    #[endpoint]
    #[payable("EGLD")]
    fn fund(&self) {
        let payment = self.call_value().egld().clone_value();

        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time < self.deadline().get(),
            "No es pot fer donacions després de la data límit"
        );

        let caller = self.blockchain().get_caller();
        let deposited_amount = self.deposit(&caller).get();
        self.deposit(&caller).set(deposited_amount + payment);
    }

    #[endpoint]
    fn claim(&self) {
        match self.status() {
            Status::FundingPeriod => sc_panic!("No es pot rescatar l'import abans de la data límit"),
            Status::Successful => {
                let caller = self.blockchain().get_caller();
                require!(
                    caller == self.blockchain().get_owner_address(),
                    "només el propietari pot recuperar les donacions"
                );

                let sc_balance = self.get_current_funds();
                self.send().direct_egld(&caller, &sc_balance);
            }
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.deposit(&caller).get();

                if deposit > 0u32 {
                    self.deposit(&caller).clear();
                    self.send().direct_egld(&caller, &deposit);
                }
            }
        }
    }

    #[view]
    fn status(&self) -> Status {
        if self.get_current_time() <= self.deadline().get() {
            Status::FundingPeriod
        } else if self.get_current_funds() >= self.target_min().get() {
            Status::Successful
        } else {
            Status::Failed
        }
    }

    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
    }

    // private
    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    // storage

    #[view(getTargetMax)]
    #[storage_mapper("target_max")]
    fn target_max(&self) -> SingleValueMapper<BigUint>;

    #[view(getTargetMin)]
    #[storage_mapper("target_min")]
    fn target_min(&self) -> SingleValueMapper<BigUint>;

    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;


}
