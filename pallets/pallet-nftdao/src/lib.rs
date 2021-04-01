#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch::{DispatchError, DispatchResult, UnfilteredDispatchable},Parameter, traits::Get};
use frame_system::ensure_signed;
use codec::{Encode,Decode};
// use sp_runtime::{
// 	DispatchResult, DispatchError, RuntimeDebug,
// 	traits::{Zero, Hash, Dispatchable, Saturating, Bounded},
// };
use sp_runtime::{
	traits::Dispatchable,
};
use sp_runtime::{traits::{AccountIdConversion, Hash}, ModuleId};
use sp_runtime::traits::BlakeTwo256;

use sp_std::vec::Vec;

use sp_core::{TypeId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"NFTDAO!!");

#[derive(Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct DAOId(pub [u8; 32]);

impl TypeId for DAOId {
	const TYPE_ID: [u8; 4] = *b"dao!";
}



/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Action: Parameter + Dispatchable<Origin=Self::Origin> + From<Call<Self>>;
	// type Action: Parameter + UnfilteredDispatchable<Origin=Self::Origin> + From<Call<Self>>;

}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as NFTDAOModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		DecodeFailed,
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

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		// #[weight = 10_000 ]
		// pub fn run(origin, data: Vec<u8>) -> dispatch::DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		// 	if let Ok(action) = T::Action::decode(&mut &data[..]) {
		// 		let ok = action.dispatch(frame_system::RawOrigin::Root.into()).is_ok();
		// 	}
		// 	Ok(())
		// }
	
		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}
	
		
	}
}


impl<T: Config> Module<T> {
	pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

	pub fn dao_id(sender_address: T::AccountId) -> T::AccountId {
		let hash = BlakeTwo256::hash(&(PALLET_ID, sender_address).encode());
		
		let id: [u8; 32] =  hash.into();
		// let DAO_ID: DAOId = DAOId(*b"123456789011");
		let dao_id: DAOId = DAOId(id);

        dao_id.into_account()
    }

    pub fn run(
        data: Vec<u8>,
    ) -> Result<bool, DispatchError> {
		if let Ok(action) = T::Action::decode(&mut &data[..]) {
			// Ok(action.dispatch(frame_system::RawOrigin::Root.into()).is_ok())
			let self_origin = frame_system::RawOrigin::Signed(Self::account_id()).into();
			// Ok(action.dispatch_bypass_filter(seld_origin).is_ok())
			Ok(action.dispatch(self_origin).is_ok())
			
		} else {
			Err(Error::<T>::DecodeFailed)?
		}
	}
}