use crate::{
    errors::Error,
    payments::state_transition::PaymentsStateTransition,
    payments::types::{
        Account, Address, CallType, PaymentReceiptData, Transaction as PaymentsTransaction,
    },
    state::VmState,
    traits::{StateMachine, StateTransition},
    types::{StateUpdate, TransactionReceipt},
};
use primitive_types::U256;
use sparse_merkle_tree::H256;

pub struct PaymentsStateMachine {
    pub state: VmState<Account>,
    stf: PaymentsStateTransition,
}

impl StateMachine<Account, PaymentsTransaction> for PaymentsStateMachine {
    fn new(root: H256) -> Self {
        let mut state = VmState::new(root);

        //TODO: Can remove get root here.
        if state.get_root() == H256::zero() {
            let mut address_in_bytes = [0u8; 32];
            let mut address2_in_bytes = [0u8; 32];

            U256::from_dec_str("1")
                .unwrap()
                .to_big_endian(&mut address_in_bytes);
            U256::from_dec_str("2")
                .unwrap()
                .to_big_endian(&mut address2_in_bytes);

            let account1 = Account {
                address: Address(address_in_bytes.into()),
                balance: 1000,
                nonce: 0,
            };
            let account2 = Account {
                address: Address(address2_in_bytes.into()),
                balance: 1000,
                nonce: 0,
            };

            state
                .update_set(vec![account1, account2])
                .expect("Init state failed.");
        }

        PaymentsStateMachine {
            state,
            stf: PaymentsStateTransition::new(),
        }
    }

    fn execute_tx(
        &mut self,
        params: PaymentsTransaction,
    ) -> Result<(StateUpdate<Account>, TransactionReceipt), Error> {
        let from_address_key = params.from.get_key();
        let to_address_key = params.to.get_key();

        let from_account: Account = match self.state.get(&from_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => panic!("Error in finding account details"),
            Ok(None) => Account {
                address: params.from.clone(),
                balance: 0,
                nonce: 0,
            },
        };

        let to_account = match self.state.get(&to_address_key) {
            Ok(Some(i)) => i,
            Err(_e) => panic!("Error in finding account details"),
            Ok(None) => Account {
                address: params.to.clone(),
                balance: 0,
                nonce: 0,
            },
        };

        let result = match self.stf.execute_tx(vec![from_account, to_account], params) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        match self.state.update_set(result.0) {
            Ok(i) => Ok((i, result.1)),
            Err(e) => Err(e),
        }
    }
}
