#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{AccountIdConversion, SaturatedConversion, Saturating},
    ModuleId,
};

use pallet_nft::NFTInterface;
use substrate_fixed::{transcendental::pow, types::I64F64};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"Exchange");

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct NonFungibleOrderInfo<AccountId, Balance> {
    pub seller: AccountId,
    pub price: Balance,
}
/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Config> as ExchangeModule {
        // Learn more about declaring storage items:
        // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        Something get(fn something): Option<u32>;
        // (collection id, token id)  => NonFungibleOrderInfo
        NonFungibleOrders get(fn nft_order): map hasher(blake2_128_concat) (T::Hash, u128) => NonFungibleOrderInfo<T::AccountId, BalanceOf<T>>;
        // sub nft collection id => ratio

    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
        Balance =
            <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, AccountId),
        // collection_id, token_id, seller, amount, price
        NonFungibleOrderCreated(Hash, u128, AccountId, u128, Balance),
        TestB(Balance),
        TestU(u128),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        NumOverflow,
        ConvertFailed,
        TokenNotFound,
        PermissionDenied,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;


        #[weight = 10_000]
        pub fn sell_nft(origin, collection_id: T::Hash, token_id: u128, amount: u128, price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::NFT::token_exist(collection_id, token_id), Error::<T>::TokenNotFound);

            let token = T::NFT::get_nft_token(collection_id, token_id);

            ensure!(&token.owner == &who, Error::<T>::PermissionDenied);

            T::NFT::_transfer_non_fungible(who.clone(), Self::account_id(), collection_id, token_id, amount)?;

            let order_info = NonFungibleOrderInfo {
                seller: who.clone(),
                price
            };

            NonFungibleOrders::<T>::insert((collection_id, token_id), order_info);

            Self::deposit_event(RawEvent::NonFungibleOrderCreated(
                collection_id,
                token_id,
                who,
                amount,
                price,
            ));

            Ok(())
        }



    }
}

// use bancor curve
// y = m * x ^ n
// r = reverseRatio  = ppm / 1000000
// after integral and simplify,
// can get these formula
// buy: p =  poolBalance * ((1 + amount / totalSupply) ** (1 / (reserveRatio)) - 1)
// sell: p = poolBalance * ( 1 - ( 1 - amount / totalSupply ) ** (1 / reserveRatio))
// current price = poolBalance / (totalSupply * reserveRatio)
// when supply is 0, p = reserveRatio * m * amount ** (1/reserveRatio)
// Thanks for the explanation in Slava Balasanov's article (https://blog.relevant.community/bonding-curves-in-depth-intuition-parametrization-d3905a681e0a)
impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
    // r  = reserve_ratio / max_weight, max_weight = 1000000, reserve_ratio >= 1
    // p = b * ((k / s + 1) ^ (n + 1) - 1)
    // n+1 => 1 / r => max_weight / reserve_ratio
    fn pow(operand: I64F64, reverse_ratio: u128) -> Result<I64F64, DispatchError> {
        // exponent = max_weight / reserve_ratio
        let max_weight = 1000000;
        if reverse_ratio == max_weight {
            return Ok(operand);
        }
        let exponent: I64F64 = I64F64::from_num(max_weight) / I64F64::from_num(reverse_ratio);
        let operand = I64F64::from_num(operand);
        let result: I64F64 = pow(operand, exponent).map_err(|_| Error::<T>::NumOverflow)?;
        Ok(result)
    }

    // buy: p =  poolBalance * ((1 + amount / totalSupply) ** (1 / (reserveRatio)) - 1)
    fn buy_cost(
        pool_balance: BalanceOf<T>,
        amount: u128,
        total_supply: u128,
        reverse_ratio: u128,
    ) -> Result<u128, DispatchError> {
        let pool_balance = pool_balance.saturated_into::<u128>();
        let one = I64F64::from_num(1);
        let operand = I64F64::from_num(amount)
            .checked_div(I64F64::from_num(total_supply))
            .ok_or(Error::<T>::NumOverflow)?;
        let operand = one.checked_add(operand).ok_or(Error::<T>::NumOverflow)?;
        let p = Self::pow(operand, reverse_ratio)?;
        let p = p.checked_sub(one).ok_or(Error::<T>::NumOverflow)?;
        let p = I64F64::from_num(pool_balance)
            .checked_mul(p)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(p.ceil().to_num::<u128>())
    }

    // sell: p = poolBalance * ( 1 - ( 1 - amount / totalSupply ) ** (1 / reserveRatio))
    fn sell_receive(
        pool_balance: BalanceOf<T>,
        amount: u128,
        total_supply: u128,
        reverse_ratio: u128,
    ) -> Result<u128, DispatchError> {
        let pool_balance = pool_balance.saturated_into::<u128>();
        let one = I64F64::from_num(1);
        let operand = I64F64::from_num(amount)
            .checked_div(I64F64::from_num(total_supply))
            .ok_or(Error::<T>::NumOverflow)?;
        let operand = one.checked_sub(operand).ok_or(Error::<T>::NumOverflow)?;
        let p = Self::pow(operand, reverse_ratio)?;
        let p = one.checked_sub(p).ok_or(Error::<T>::NumOverflow)?;
        let p = I64F64::from_num(pool_balance)
            .checked_mul(p)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(p.to_num::<u128>())
    }
}
