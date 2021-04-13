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
use frame_system::{self as system, ensure_signed};
use pallet_collection::{CollectionInterface, TokenType};
use pallet_nft::NFTInterface;
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedMul, SaturatedConversion, Saturating, Zero},
    ModuleId,
};
use substrate_fixed::{transcendental::pow, types::I32F32, types::I64F64};

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
    pub amount: u128,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct SemiFungiblePoolInfo<AccountId, Balance, BlockNumber> {
    pub seller: AccountId,
    pub supply: u128,
    pub m: u128,
    pub sold: u128,
    pub reverse_ratio: u128,
    pub pool_balance: Balance,
    pub end_time: BlockNumber,
    // pub start_block_number: BlockNumber,
    // pub duration: BlockNumber,
}
/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Collection: CollectionInterface<Self::Hash, Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
}

decl_storage! {

    trait Store for Module<T: Config> as ExchangeModule {
        // (collection id, token id)  => NonFungibleOrderInfo
        NonFungibleOrders get(fn nft_order): map hasher(blake2_128_concat) (T::Hash, u128) => NonFungibleOrderInfo<T::AccountId, BalanceOf<T>>;
        // (collection id, seller_account) => pool
        SemiFungiblePools get (fn semi_fungible_pool): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => SemiFungiblePoolInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
        BlockNumber = <T as frame_system::Config>::BlockNumber,
        Balance =
            <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
    {
        SomethingStored(u32, AccountId),
        // collection_id, token_id, seller, amount, price
        NonFungibleOrderCreated(Hash, u128, AccountId, u128, Balance),
        // collection_id, token_id
        NonFungibleOrderCanceled(Hash, u128),
        // collection_id, token_id, buyer, amount)
        NonFungibleSold(Hash, u128, AccountId, u128),
        // collection_id, seller, amount, reverse_ratio, m, end_time
        SemiFungiblePoolCreated(Hash, AccountId, u128, u128, u128, BlockNumber),
        // collection_id, seller, buyer, amount, cost
        SemiFungibleSold(Hash, AccountId, AccountId, u128, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NumOverflow,
        ConvertFailed,
        CollectionNotFound,
        TokenNotFound,
        OrderNotFound,
        PoolNotFound,
        PoolExisted,
        AmountTooLarge,
        AmountLessThanOne,
        PermissionDenied,
        WrongTokenType,
        ExpiredSoldTime
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

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
                price,
                amount
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

        #[weight = 10_000]
        pub fn buy_nft(origin, collection_id: T::Hash, token_id: u128, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
            ensure!(NonFungibleOrders::<T>::contains_key((collection_id, token_id)), Error::<T>::OrderNotFound);

            let order = Self::nft_order((collection_id, token_id));

            ensure!(&order.amount >= &amount, Error::<T>::AmountTooLarge);

            let price = &order.price;
            let b_amout = amount.saturated_into::<BalanceOf<T>>();
            let cost = price.checked_mul(&b_amout).ok_or(Error::<T>::NumOverflow)?;
            let left_amount = &order.amount.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            T::Currency::transfer(&who, &order.seller, cost, AllowDeath)?;
            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), collection_id, token_id, amount)?;

            if left_amount.clone() == 0 {
                NonFungibleOrders::<T>::remove((collection_id, token_id));
            } else {
                let order = NonFungibleOrderInfo {
                    amount: *left_amount,
                    ..order
                };
                NonFungibleOrders::<T>::insert((collection_id, token_id), order);
            }

            Self::deposit_event(RawEvent::NonFungibleSold(
                collection_id,
                token_id,
                who,
                amount,
            ));

            Ok(())
        }

        #[weight = 10_000]
        pub fn cancel_nft_order(origin, collection_id: T::Hash, token_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(NonFungibleOrders::<T>::contains_key((collection_id, token_id)), Error::<T>::OrderNotFound);

            let order = Self::nft_order((collection_id, token_id));

            ensure!(&order.seller == &who, Error::<T>::PermissionDenied);

            let amount = &order.amount;

            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), collection_id, token_id, *amount)?;
            NonFungibleOrders::<T>::remove((collection_id, token_id));

            Self::deposit_event(RawEvent::NonFungibleOrderCanceled(
                collection_id,
                token_id,
            ));

            Ok(())
        }

        #[weight = 10_000]
        pub fn create_semi_token_pool(origin, collection_id: T::Hash, amount: u128, reverse_ratio: u128, m: u128, duration: T::BlockNumber) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
            // if pool existed, withdraw and delete pool
            ensure!(!SemiFungiblePools::<T>::contains_key((&collection_id, &who)), Error::<T>::PoolExisted);
            ensure!(T::Collection::collection_exist(collection_id.clone()), Error::<T>::CollectionNotFound);
            ensure!(T::NFT::get_balance(&collection_id, &who) >= amount, Error::<T>::AmountTooLarge);

            let collection = T::Collection::get_collection(collection_id.clone());
            if let Some(token_type) = collection.token_type {
                ensure!(
                    token_type == TokenType::Fungible,
                    Error::<T>::WrongTokenType
                );
            }

            let block_number = <system::Pallet<T>>::block_number();
            let end_time = block_number.checked_add(&duration).ok_or(Error::<T>::NumOverflow)?;

            let pool = SemiFungiblePoolInfo {
                m,
                reverse_ratio,
                end_time,
                sold: 0,
                seller: who.clone(),
                supply: amount,
                pool_balance: 0_u128.saturated_into::<BalanceOf<T>>(),
            };

            T::NFT::_transfer_fungible(who.clone(), Self::account_id(), collection_id.clone(), amount)?;
            SemiFungiblePools::<T>::insert((&collection_id, &who), pool);

            Self::deposit_event(RawEvent::SemiFungiblePoolCreated(
                collection_id,
                who,
                amount,
                reverse_ratio,
                m,
                end_time
            ));

            Ok(())
        }

        #[weight = 10_000]
        pub fn buy_semi_token(origin, collection_id: T::Hash, seller: T::AccountId, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(SemiFungiblePools::<T>::contains_key((&collection_id, &seller)), Error::<T>::PoolNotFound);

            let pool = Self::semi_fungible_pool((&collection_id, &seller));

            let reverse_ratio = &pool.reverse_ratio;
            let total_supply = &pool.sold;

            ensure!(&amount <= &pool.supply, Error::<T>::AmountTooLarge);

            let block_number = <system::Pallet<T>>::block_number();
            ensure!(&block_number <= &pool.end_time, Error::<T>::ExpiredSoldTime);

            let cost = if &pool.sold == & 0 {
                let m = &pool.m;
                Self::first_buy_cost(*reverse_ratio, *m, amount)?
             } else {
                let pool_balance = &pool.pool_balance;
                Self::buy_cost(*pool_balance, amount, *total_supply, *reverse_ratio)?
            };

            let cost = cost.saturated_into::<BalanceOf<T>>();

            let sold = &pool.sold.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            let supply = &pool.supply.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            let pool_balance = pool.pool_balance.clone().checked_add(&cost).ok_or(Error::<T>::NumOverflow)?;

            let pool = SemiFungiblePoolInfo {
                sold: *sold,
                supply: *supply,
                pool_balance,
                ..pool
            };

            SemiFungiblePools::<T>::insert((&collection_id, &seller), pool);

            Self::deposit_event(RawEvent::SemiFungibleSold(
                collection_id,
                seller,
                who,
                amount,
                cost
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

    fn to_fixed2(operand: I64F64) -> Result<I64F64, DispatchError> {
        let hundred = I64F64::from_num(100);
        let r = operand
            .checked_mul(hundred)
            .ok_or(Error::<T>::NumOverflow)?;

        Ok(r.round() / 100)
    }

    fn first_buy_cost(reverse_ratio: u128, m: u128, amount: u128) -> Result<u128, DispatchError> {
        // r  = reserve_ratio / max_weight
        // p = r * m * amount ** (1/r)
        let max_weight = I64F64::from_num(1000000);
        let m = I64F64::from_num(m);
        let one = I64F64::from_num(1);
        let amount = I64F64::from_num(amount);

        let r: I64F64 = I64F64::from_num(reverse_ratio) / max_weight;

        let exponent: I64F64 = I64F64::from_num(max_weight) / I64F64::from_num(reverse_ratio);
        let operand: I64F64 = pow(amount, exponent).map_err(|_| Error::<T>::NumOverflow)?;

        let operand = operand.checked_mul(m).ok_or(Error::<T>::NumOverflow)?;
        let p = operand.checked_mul(r).ok_or(Error::<T>::NumOverflow)?;

        let p = Self::to_fixed2(p)?;

        Ok(p.ceil().to_num::<u128>())
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
        let p = Self::to_fixed2(p)?;
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
