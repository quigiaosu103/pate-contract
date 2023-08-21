#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod phat_hello {

    use ink::{prelude::string::String, codegen::StaticEnv};
    use ink::prelude::vec::Vec;
    use scale::{Decode, Encode, EncodeLike};
    use serde::Deserialize;
    use ink::storage::Mapping;
    use ink::storage::traits::Packed;


    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct PromptNFT {
        id: String,
        title: String,
        owner: String,
        type_: String,
        price: Balance,
        prompt_id: String
    }

    #[ink(event)]
    pub struct Transferred {
        from: Option<AccountId>,
        to: Option<String>,
        value: Balance,
    }
    
    #[ink(storage)]
    pub struct PromptMarketplaceContract {
        base_name: String,
        nfts: Vec<String>,
        nft_by_id: Mapping<String, (String, String, String, Balance, String)>,
        nfts_by_owner: Mapping<String, Vec<String>>
    }

    impl PromptMarketplaceContract {
        /// Constructor to initializes
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                base_name: String::from("prompt marketplace"),
                nfts: Vec::new(),
                nft_by_id: Mapping::new(),
                nfts_by_owner: Mapping::new()
            }
        }

        #[ink(message)]
        pub fn new_prompt(&mut self, id: String, title: String, caller: String, type_: String, price: Balance, prompt_id: String) -> PromptNFT {
            let prompt = PromptNFT {
                id: id.clone(),
                title: title.clone(),
                owner: caller.clone(),
                type_: type_.clone(),
                price: price.clone(),
                prompt_id: prompt_id.clone()
            };
            let mut vec_nfts = self.nfts_by_owner.get(id.clone()).unwrap_or_else(|| Vec::new());
            vec_nfts.push(id.clone());
            self.nfts.push(id.clone());
            self.nft_by_id.insert(id.clone(), &(title, caller, type_, 1, prompt_id));
            self.nfts_by_owner.insert(id, &vec_nfts);
            prompt

        }

        #[ink(message)]
        pub fn get_prompt_by_id(&self, key: String) -> PromptNFT {
            let a = self.nft_by_id.get(key.clone()).unwrap();
            PromptNFT {
                id: key,
                title: a.0,
                owner: a.1,
                type_: a.2,
                price: a.3,
                prompt_id: a.4,
            }
        }
        
        #[ink(message)]
        pub fn get_all_prompts(&self) -> Vec<PromptNFT> {
            let mut nfts: Vec<PromptNFT> = Vec::new();
            let ids = &self.nfts;
            for i in ids{
                let nft = self.nft_by_id.get(i.clone()).unwrap();
                nfts.push(PromptNFT {
                    id: String::from(i),
                    title: nft.0,
                    owner: nft.1,
                    type_: nft.2,
                    price: nft.3,
                    prompt_id: nft.4
                })
            }
            nfts
        }

        #[ink(message)]
        pub fn update_price(&mut self, id: String, price: Balance) -> String{
            let nft  = self.nft_by_id.get(id.clone()).unwrap();
            self.nft_by_id.insert(id.clone(), &(nft.0, nft.1, nft.2, price, nft.4));
            String::from("update price prompt_id: {id}, price: {price}")
        }

        #[ink(message)]
        pub fn get_prompts_by_owner(&self, owner_id: String) -> Vec<PromptNFT> {
            let mut vec_nfts: Vec<PromptNFT> = Vec::new();
            let vec_ids = self.nfts_by_owner.get(owner_id).unwrap_or_else(|| Vec::new());
            for i in vec_ids {
                let nft = self.nft_by_id.get(i.clone()).unwrap();
                vec_nfts.push(PromptNFT {
                    id: String::from(i),
                    title: nft.0,
                    owner: nft.1,
                    type_: nft.2,
                    price: nft.3,
                    prompt_id: nft.4
                });
            }
            vec_nfts
        } 

        #[ink(message, payable)]
        pub fn payment(&self, nft_id: String) {
            let caller = self.env().caller();
            assert!(self.nft_by_id.contains(nft_id.clone()), "Prompt is not valid!");
            let nft = self.nft_by_id.get(nft_id).unwrap();
            let owner = nft.1;
            self.env().emit_event(Transferred {
                from: Some(caller),
                to: Some(owner),
                value: nft.3,
            });
        }


    }

}
