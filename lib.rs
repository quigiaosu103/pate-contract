#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod phat_hello {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use pink_extension::{http_post, http_get};

    // use ink_env::account_balance;



    #[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotFetchValue,
        NotAllowed,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct PromptNFT {
        id: String,
        title: String,
        owner: AccountId,
        type_: String,
        price: Balance,
        prompt_id: String
    }

    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    #[ink(event)]
    pub struct Transferred {
        from: Option<AccountId>,
        to: Option<AccountId>,
        sattus: String,
        value: Balance,
    }
    
    #[ink(storage)]
    pub struct PromptMarketplaceContract {
        base_name: String,
        nfts: Vec<String>,
        nft_by_id: Mapping<String, (String, AccountId, String, Balance, String)>,
        nfts_by_owner: Mapping<AccountId, Vec<String>>
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
        pub fn new_prompt(&mut self, id: String, title: String, type_: String, price: Balance, prompt_content: String) -> PromptNFT {
            let caller = self.env().caller();
            let prompt = PromptNFT {
                id: id.clone(),
                title: title.clone(),
                owner: caller.clone(),
                type_: type_.clone(),
                price: price.clone(),
                prompt_id: prompt_content.clone()
            };
            let mut vec_nfts = self.nfts_by_owner.get(caller.clone()).unwrap_or_else(|| Vec::new());
            vec_nfts.push(id.clone());
            self.nfts.push(id.clone());
            self.nft_by_id.insert(id.clone(), &(title, caller, type_, price, prompt_content));
            self.nfts_by_owner.insert(caller, &vec_nfts);
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
        pub fn get_prompts_by_owner(&self, owner_id: AccountId) -> Vec<PromptNFT> {
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
        pub fn payment(&mut self, nft_id: String) -> Result<(), Error> {
            let caller = self.env().caller();
            let nft = self.nft_by_id.get(nft_id.clone()).unwrap();
            let owner = nft.1;
            let deposit = self.env().balance();
            assert!(self.nft_by_id.contains(nft_id.clone()), "Prompt is not valid!");
            assert_eq!(deposit, nft.3, "wrong deposit!");
            self.env().emit_event(Transferred {
                from: Some(caller.clone()),
                to: Some(owner.clone()),
                sattus: String::from("Successful"),
                value: nft.3,
            });
            self.env().transfer(owner, nft.3).expect("transfer failed");
            self.nft_by_id.insert(nft_id, &(nft.0, caller, nft.2, nft.3, nft.4));
            Ok(())
        }

        // #[ink(message)]
        // pub fn request(&self) -> String {
        //     let mut headers: Vec<(String, String)> = Vec::new();
        //     headers.push((String::from("Content-Type"), String::from("application/json")));
        //     headers.push((String::from("Authorization"), String::from("Bearer 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")));
        //     // let headers = vec![
        //     //     ("Content-Type".to_string(), "application/json".to_string()),
        //     //     ("Authorization".to_string(), format!("Bearer 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")),
        //     // ];
            
        //     let data = r#"
        //     {
        //         "prompt": "a white siamese cat",
        //         "n": 1,
        //         "size": "1024x1024"
        //     }
        //     "#;

        //     let response = http_post!("https://api.openai.com/v1/images/generations", data, headers);
        //     let a = response.reason_phrase;
        //     // Process the response as needed
        //     a
        // }
        #[ink(message)]
        pub fn get_request_status_code(&self) -> u16 {
            let mut headers: Vec<(String, String)> = Vec::new();
            headers.push((String::from("Content-Type"), String::from("application/json")));
            headers.push((String::from("Authorization"), String::from("Bearer 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")));
            // let headers = vec![
            //     ("Content-Type".to_string(), "application/json".to_string()),
            //     ("Authorization".to_string(), format!("Bearer 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")),
            // ];
            
            let data = r#"
            {
                "prompt": "a white siamese cat",
                "n": 1,
                "size": "1024x1024"
            }
            "#;

            let response: pink_extension::chain_extension::HttpResponse = http_get!("https://jsonplaceholder.typicode.com/todos/1");
            let a = response.status_code;
            // Process the response as needed
            a
        }
        
        #[ink(message)]
        pub fn http_get_example(&self) {
            let response = http_get!("https://jsonplaceholder.typicode.com/todos/1");
            assert_eq!(response.status_code, 200);
        }

        #[ink(message)]
        pub fn get_request(&self) -> Vec<u8>{
            let mut headers: Vec<(String, String)> = Vec::new();
            headers.push((String::from("Content-Type"), String::from("application/json")));
            headers.push((String::from("Authorization"), String::from("Bearer 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")));
            // let headers = vec![
            //     ("Content-Type".to_string(), "application/json".to_string()),
            //     ("Authorization".to_string(), format!("Bearer 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")),
            // ];
            
            let data = r#"
            {
                "prompt": "a white siamese cat",
                "n": 1,
                "size": "1024x1024"
            }
            "#;

            let response = http_get!("https://jsonplaceholder.typicode.com/todos/1");
            let a = response.body;
            // Process the response as needed
            a
        }

    }

}
