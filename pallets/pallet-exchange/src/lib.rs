//! # Exchange Module
//!
//! Exchange NFTs or FTs.
//!
//! For pool, use bancor curve.
//! y = m * x ^ n
//! r = reverseRatio  = ppm / 1000000
//! after integral and simplify,
//! can get these formula
//! buy: p =  poolBalance * ((1 + amount / totalSupply) ** (1 / (reserveRatio)) - 1)
//! sell: p = poolBalance * ( 1 - ( 1 - amount / totalSupply ) ** (1 / reserveRatio))
//! current price = poolBalance / (totalSupply * reserveRatio)
//! when supply is 0, p = reserveRatio * m * amount ** (1/reserveRatio)
//! Thanks for the explanation in Slava Balasanov's article (https://blog.relevant.community/bonding-curves-in-depth-intuition-parametrization-d3905a681e0a)
//!
//! ### Terminology
//!
//! * **Pool:** It can be exchanged with some FTs, and the price can be automatically discovered through bancor curve.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `sell_nft` - Sell one or a batch of NFTs.
//! * `buy_nft` - Buy one or a batch of NFTs.
//! * `cancel_nft_order` - Cancel the order and get back the NFTs locked in the pallet.
//! * `create_semi_token_pool` - Create a time-limited pool.
//! * `sell_semi_token` - Sell FTs to pool.
//! * `withdraw_pool` - After the time of the pool has passed, the creator of the pool can obtain the assets in the pool.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

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
};
use frame_system::{self as system, ensure_signed};
use pallet_collection::{CollectionInterface, TokenType};
use pallet_nft::NFTInterface;
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedMul, CheckedSub, SaturatedConversion},
    ModuleId,
};
use substrate_fixed::{transcendental::pow, types::I64F64};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"Exchange");

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

///Order details.
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct NonFungibleOrderInfo<Hash, AccountId, Balance> {
    pub collection_id: Hash,
    pub start_idx: u128,
    pub seller: AccountId,
    pub price: Balance,
    pub amount: u128,
}

/// Pool details.
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct SemiFungiblePoolInfo<AccountId, Balance, BlockNumber> {
    pub seller: AccountId,
    pub supply: u128,
    pub m: u128,
    pub sold: u128,
    pub reverse_ratio: u128,
    pub pool_balance: Balance,
    pub end_time: BlockNumber,
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
        /// Next order_id.
        NextNonFungibleOrderId get(fn next_nft_order_id): u128 = 0;
        /// The set of NFT orders. order_id => (collection_id, token_id)
        NonFungibleOrders get(fn nft_order): map hasher(blake2_128_concat) u128 => NonFungibleOrderInfo<T::Hash, T::AccountId, BalanceOf<T>>;
        // The set of FTs pools. (collection id, seller_account) => pool
        SemiFungiblePools get (fn semi_fungible_pool): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => SemiFungiblePoolInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>;
    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
        BlockNumber = <T as frame_system::Config>::BlockNumber,
        Balance =
            <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
    {
        /// An NFT order was created. \[order_id\]
        NonFungibleOrderCreated(u128),
        /// An NFT order was calceled. \[nft_order_id\]
        NonFungibleOrderCanceled(u128),
        ///  One or a batch of NFTs were sold. \[amount\]
        NonFungibleSold(u128),
        /// A pool was created. \[ft_collection_id\]
        SemiFungiblePoolCreated(Hash),
        /// assets were taken out from pool \[collection_id, seller\]
        SemiFungiblePoolWithdrew(Hash, AccountId),
        /// Buy FTs from pool. \[collection_id, cost\]
        SemiFungibleBought(Hash, Balance),
        /// Sell FTs to pool. \[collection_id, receive\]
        SemiFungibleSold(Hash, Balance),
    }
);

decl_error! {
    /// Errors inform users that something went wrong.
    pub enum Error for Module<T: Config> {
        /// Number is too large or less than zero.
        NumOverflow,
        /// Collection does not exist.
        CollectionNotFound,
        /// Token does not exist.
        TokenNotFound,
        /// Order does not exist.
        OrderNotFound,
        /// Pool does not exist.
        PoolNotFound,
        /// Only one pool of FTs of the same collection can exist.
        PoolExisted,
        /// Amount is more than own.
        AmountTooLarge,
        /// The minimum value of amount is 1.
        AmountLessThanOne,
        /// The minimum value of reverse_ratio is 1.
        ReverseRatioLessThanOne,
        /// The minimum value of m is 1.
        MLessThanOne,
        /// No permission to perform this operation.
        PermissionDenied,
        /// Wrong token type.
        WrongTokenType,
        /// Can't Sell FTs now.
        ExpiredSoldTime,
        /// Not yet crossed the duration of the pool.
        CanNotWithdraw,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Sell one or a batch of NFTs.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection_id of the NFT to be sold.
        /// - `token_id`: The index of the NFT to be sold.
        /// - `amount`: How many NFTs to sell.
        /// - `price`: Pricing of NFTs.
        #[weight = 10_000]
        pub fn sell_nft(origin, collection_id: T::Hash, token_id: u128, amount: u128, price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::NFT::token_exist(collection_id, token_id), Error::<T>::TokenNotFound);

            let token = T::NFT::get_nft_token(collection_id, token_id);

            ensure!(&token.owner == &who, Error::<T>::PermissionDenied);

            let nft_order_id = Self::next_nft_order_id();
            let next_nft_order_id = nft_order_id.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            T::NFT::_transfer_non_fungible(who.clone(), Self::account_id(), collection_id, token_id, amount)?;

            let order_info = NonFungibleOrderInfo {
                collection_id: collection_id.clone(),
                start_idx: token_id,
                seller: who.clone(),
                price,
                amount
            };

            NonFungibleOrders::<T>::insert(nft_order_id, order_info);
            NextNonFungibleOrderId::put(next_nft_order_id);

            Self::deposit_event(RawEvent::NonFungibleOrderCreated(
                nft_order_id
            ));

            Ok(())
        }

        /// Buy one or a batch of NFTs.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `order_id`: The id of the order
        /// - `amount`: How many NFTs to buy.
        #[weight = 10_000]
        pub fn buy_nft(origin, order_id: u128, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
            ensure!(NonFungibleOrders::<T>::contains_key(order_id), Error::<T>::OrderNotFound);

            let order = Self::nft_order(order_id);

            ensure!(&order.amount >= &amount, Error::<T>::AmountTooLarge);

            let price = &order.price;
            let b_amout = amount.saturated_into::<BalanceOf<T>>();
            let cost = price.checked_mul(&b_amout).ok_or(Error::<T>::NumOverflow)?;
            let left_amount = &order.amount.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            let collection_id = &order.collection_id;
            let token_id = &order.start_idx;

            T::Currency::transfer(&who, &order.seller, cost, AllowDeath)?;
            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), *collection_id, *token_id, amount)?;

            // let sended_token = T::NFT::get_nft_token(collection_id.clone(), token_id.clone());
            // let start_idx = sended_token.end_idx.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            // if sold out, remove order
            if *left_amount == 0 {
                NonFungibleOrders::<T>::remove(order_id);
            } else {
                let start_idx = token_id.checked_add(&amount).ok_or(Error::<T>::NumOverflow)?;
                let order = NonFungibleOrderInfo {
                    amount: *left_amount,
                    start_idx,
                    ..order
                };
                NonFungibleOrders::<T>::insert(order_id, order);
            }

            Self::deposit_event(RawEvent::NonFungibleSold(
                *left_amount
            ));

            Ok(())
        }

        /// Cancel the order and get back the NFTs locked in the pallet.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `order_id`: Order to cancel.
        #[weight = 10_000]
        pub fn cancel_nft_order(origin, order_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(NonFungibleOrders::<T>::contains_key(order_id), Error::<T>::OrderNotFound);

            let order = Self::nft_order(order_id);

            ensure!(&order.seller == &who, Error::<T>::PermissionDenied);

            let amount = &order.amount;
            let collection_id = &order.collection_id;
            let token_id = &order.start_idx;

            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), *collection_id, *token_id, *amount)?;
            NonFungibleOrders::<T>::remove(order_id);

            Self::deposit_event(RawEvent::NonFungibleOrderCanceled(order_id));

            Ok(())
        }

        /// Create a pool.
        ///
        /// Because the Bancor formula cannot be used when the assets in the pool are zero,
        /// you need to use m to calculate the price of the first purchase.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection where FT is located.
        /// - `amount`: How many FTs to sell.
        /// - `reverse_ratio`: Value that affects price sensitivity.
        /// - `duration`: The duration of the pool.
        #[weight = 10_000]
        pub fn create_semi_token_pool(origin, collection_id: T::Hash, amount: u128, reverse_ratio: u128, m: u128, duration: T::BlockNumber) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(reverse_ratio >=1, Error::<T>::ReverseRatioLessThanOne);
            ensure!(m >=1, Error::<T>::MLessThanOne);

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
                collection_id
            ));

            Ok(())
        }

        /// Buy FTs from pool.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection where FT is located.
        /// - `seller`: The creator of the pool.
        /// - `amount`: How many NFTs to buy.
        #[weight = 10_000]
        pub fn buy_semi_token(origin, collection_id: T::Hash, seller: T::AccountId, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(SemiFungiblePools::<T>::contains_key((&collection_id, &seller)), Error::<T>::PoolNotFound);

            let pool = Self::semi_fungible_pool((&collection_id, &seller));

            ensure!(&amount <= &pool.supply, Error::<T>::AmountTooLarge);

            let block_number = <system::Pallet<T>>::block_number();
            ensure!(&block_number <= &pool.end_time, Error::<T>::ExpiredSoldTime);

            let reverse_ratio = &pool.reverse_ratio;
            let total_supply = &pool.sold;

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

            T::Currency::transfer(&who, &Self::account_id(), cost, AllowDeath)?;
            T::NFT::_transfer_fungible(Self::account_id(), who.clone(), collection_id.clone(), amount)?;

            SemiFungiblePools::<T>::insert((&collection_id, &seller), pool);

            Self::deposit_event(RawEvent::SemiFungibleBought(
                collection_id,
                cost
            ));

            Ok(())
        }

        /// Sell FTs to pool.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection where FT is located.
        /// - `seller`: The creator of the pool.
        /// - `amount`: How many NFTs to sell.
        #[weight = 10_000]
        pub fn sell_semi_token(origin, collection_id: T::Hash, seller: T::AccountId, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let pool_id = (&collection_id, &seller);
            ensure!(SemiFungiblePools::<T>::contains_key(pool_id), Error::<T>::PoolNotFound);

            let pool = Self::semi_fungible_pool((&collection_id, &seller));

            // pool.sold should large than 0
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
            ensure!(&pool.sold >= &amount, Error::<T>::AmountTooLarge);

            let block_number = <system::Pallet<T>>::block_number();
            ensure!(&block_number <= &pool.end_time, Error::<T>::ExpiredSoldTime);

            let reverse_ratio = &pool.reverse_ratio;
            let total_supply = &pool.sold;
            let pool_balance = &pool.pool_balance;

            let receive = Self::sell_receive(*pool_balance, amount, *total_supply, *reverse_ratio)?;
            let receive = receive.saturated_into::<BalanceOf<T>>();

            let new_pool_balance = pool.pool_balance.clone().checked_sub(&receive).ok_or(Error::<T>::NumOverflow)?;
            let sold = &pool.sold.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
            let supply = &pool.supply.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;

            let pool = SemiFungiblePoolInfo {
                sold: *sold,
                supply: *supply,
                pool_balance: new_pool_balance,
                ..pool
            };

            T::NFT::_transfer_fungible(who.clone(), Self::account_id(), collection_id.clone(), amount)?;
            T::Currency::transfer(&Self::account_id(), &who, receive, AllowDeath)?;

            SemiFungiblePools::<T>::insert(pool_id, pool);

            Self::deposit_event(RawEvent::SemiFungibleSold(
                collection_id,
                receive
            ));

            Ok(())
        }

        /// Withdraw assets from the pool by creator.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection where FT is located.
        #[weight = 10_000]
        pub fn withdraw_pool(origin, collection_id: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let pool_id = (&collection_id, &who);
            ensure!(SemiFungiblePools::<T>::contains_key(pool_id), Error::<T>::PoolNotFound);

            let pool = Self::semi_fungible_pool((&collection_id, &who));
            ensure!(&pool.seller == &who, Error::<T>::PermissionDenied);

            let block_number = <system::Pallet<T>>::block_number();
            ensure!(&block_number > &pool.end_time, Error::<T>::CanNotWithdraw);

            let pool_balance = &pool.pool_balance;
            let supply = &pool.supply;

            SemiFungiblePools::<T>::remove(pool_id);
            T::NFT::_transfer_fungible(Self::account_id(), who.clone(), collection_id.clone(), *supply)?;
            T::Currency::transfer(&Self::account_id(), &who, *pool_balance, AllowDeath)?;

            Self::deposit_event(RawEvent::SemiFungiblePoolWithdrew(
                collection_id,
                who,
            ));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Account of this pallet.
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    /// pow operation
    /// r  = reserve_ratio / max_weight, max_weight = 1000000, reserve_ratio >= 1
    /// p = b * ((k / s + 1) ^ (n + 1) - 1)
    /// n+1 => 1 / r => max_weight / reserve_ratio
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

    /// Keep two decimal places
    fn to_fixed2(operand: I64F64) -> Result<I64F64, DispatchError> {
        let hundred = I64F64::from_num(100);
        let r = operand
            .checked_mul(hundred)
            .ok_or(Error::<T>::NumOverflow)?;

        Ok(r.round() / 100)
    }

    /// Calculate the price of the first purchase
    /// r  = reserve_ratio / max_weight
    /// p = r * m * amount ** (1/r)
    fn first_buy_cost(reverse_ratio: u128, m: u128, amount: u128) -> Result<u128, DispatchError> {
        
        let max_weight = I64F64::from_num(1000000);
        let m = I64F64::from_num(m);
        let amount = I64F64::from_num(amount);

        let r: I64F64 = I64F64::from_num(reverse_ratio) / max_weight;

        let exponent: I64F64 = I64F64::from_num(max_weight) / I64F64::from_num(reverse_ratio);
        let operand: I64F64 = pow(amount, exponent).map_err(|_| Error::<T>::NumOverflow)?;

        let operand = operand.checked_mul(m).ok_or(Error::<T>::NumOverflow)?;
        let p = operand.checked_mul(r).ok_or(Error::<T>::NumOverflow)?;

        let p = Self::to_fixed2(p)?;

        Ok(p.ceil().to_num::<u128>())
    }

    /// Get the price of purchasing FT
    /// buy: p =  poolBalance * ((1 + amount / totalSupply) ** (1 / (reserveRatio)) - 1)
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

    /// Price of selling FT
    /// sell: p = poolBalance * ( 1 - ( 1 - amount / totalSupply ) ** (1 / reserveRatio))
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
