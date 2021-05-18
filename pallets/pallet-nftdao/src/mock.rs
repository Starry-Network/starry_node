use crate as pallet_dao;
use frame_support::parameter_types;
use frame_support::traits::TestRandomness;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
use pallet_collection::CollectionInterface;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Template: pallet_template::{Module, Call, Storage, Event<T>},
        CollectionModule: pallet_collection::{Module, Call, Storage, Event<T>},
        NFTModule: pallet_nft::{Module, Call, Storage, Event<T>},
        DaoModule: pallet_dao::{Module, Call, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

impl pallet_template::Config for Test {
    type Event = Event;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl pallet_collection::Config for Test {
    type Event = Event;
    type RandomnessSource = TestRandomness;
}

impl pallet_nft::Config for Test {
    type Event = Event;
    type Collection = CollectionModule;
}

impl pallet_dao::Config for Test {
    type Event = Event;
    type Action = Call;
    type RandomnessSource = TestRandomness;
    type Currency = Balances;
    type NFT = NFTModule;
}

pub const PERIOD_DURATION: u128 = 2;
pub const WRONG_PERIOD_DURATION: u128 = 0;
pub const VOTING_PERIOD: u128 = 1;
pub const WRONG_VOTING_PERIOD: u128 = 0;
pub const GRACE_PERIOD: u128 = 1;
pub const WRONG_GRACE_PERIOD: u128 = 0;
pub const METADATA: Vec<u8> = Vec::new();
pub const SHARES_REQUESTED: u128 = 1;
pub const PROPOSAL_DEPOSIT: u64 = 1;
pub const WRONG_PROPOSAL_DEPOSIT: u64 = 0;
pub const PROCESSING_REWARD: u64 = 1;
pub const DILUTION_BOUND: u128 = 3;
pub const WRONG_DILUTION_BOUND: u128 = 0;

pub fn mint_a_nft(minter_address: &u64) -> (H256, u128) {
    let minter = Origin::signed(*minter_address);
    CollectionModule::create_collection(minter.clone(), vec![2, 3, 3], false).unwrap();
    let nonce = CollectionModule::get_nonce();
    let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

    NFTModule::mint_non_fungible(
        minter,
        *minter_address,
        collection_id.clone(),
        vec![2, 3, 3],
        1
    ).unwrap();

    (collection_id, 0)
}

pub fn get_last_dao_account(summoner_address: &u64, name: &Vec<u8>) -> u64 {
    let nonce = DaoModule::get_nonce();
    let id = DaoModule::_dao_id(&summoner_address, &name, nonce);
    let dao_id = crate::DAOId(id);
    DaoModule::dao_account_id(&dao_id)
}

pub fn create_a_dao(summoner_address: &u64, proposal_deposit: u64, proposal_reward: u64) -> u64 {
    let summoner = Origin::signed(*summoner_address);

    DaoModule::create_dao(
        summoner,
        METADATA,
        PERIOD_DURATION,
        VOTING_PERIOD,
        GRACE_PERIOD,
        SHARES_REQUESTED,
        proposal_deposit,
        proposal_reward,
        DILUTION_BOUND
    ).unwrap();
    get_last_dao_account(&summoner_address, &METADATA)
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
