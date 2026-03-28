#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if decimal > 18 {
            panic!("Decimal must not be greater than 18");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Decimal, &decimal);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let total_supply: i128 = env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0);
        let new_total_supply = total_supply.checked_add(amount).expect("Overflow");
        env.storage().instance().set(&DataKey::TotalSupply, &new_total_supply);

        let balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        let new_balance = balance.checked_add(amount).expect("Overflow");
        env.storage().persistent().set(&DataKey::Balance(to), &new_balance);

        env.events().publish((Symbol::new(&env, "mint"), to), amount);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let new_balance = balance.checked_sub(amount).expect("Insufficient balance");
        env.storage().persistent().set(&DataKey::Balance(from), &new_balance);

        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        let new_to_balance = to_balance.checked_add(amount).expect("Overflow");
        env.storage().persistent().set(&DataKey::Balance(to.clone()), &new_to_balance);

        env.events().publish((Symbol::new(&env, "transfer"), from, to), amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        let allowance_key = AllowanceDataKey {
            from: from.clone(),
            spender: spender.clone(),
        };
        let allowance: AllowanceValue = env.storage().temporary().get(&allowance_key).unwrap_or(AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        });

        if allowance.expiration_ledger < env.ledger().sequence() {
            panic!("Allowance expired");
        }

        let new_allowance_amount = allowance.amount.checked_sub(amount).expect("Insufficient allowance");
        let new_allowance = AllowanceValue {
            amount: new_allowance_amount,
            expiration_ledger: allowance.expiration_ledger,
        };
        env.storage().temporary().set(&allowance_key, &new_allowance);

        Self::transfer(env, from, to, amount);
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        let allowance_key = AllowanceDataKey {
            from,
            spender,
        };
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };
        env.storage().temporary().set(&allowance_key, &allowance);

        env.events().publish((Symbol::new(&env, "approve"), from, spender), (amount, expiration_ledger));
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let allowance_key = AllowanceDataKey { from, spender };
        env.storage().temporary().get(&allowance_key).unwrap_or(AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }).amount
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(id)).unwrap_or(0)
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Decimal).unwrap()
    }

    pub fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap()
    }

    pub fn symbol(env: Env) -> String {
        env.storage().instance().get(&DataKey::Symbol).unwrap()
    }

    pub fn total_supply(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0)
    }
}

#[contracttype]
pub enum DataKey {
    Admin,
    Decimal,
    Name,
    Symbol,
    TotalSupply,
    Balance(Address),
}