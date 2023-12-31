#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;
use pink_extension as pink;
mod utils;

#[pink::contract(env=PinkEnvironment)]
mod phavault {
    use super::pink::PinkEnvironment;
    use crate::utils::*;
    use alloc::borrow::ToOwned;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use argon2::{Algorithm, Argon2, Params, Version};
    use bin_serde;
    use chacha20poly1305::{
        aead::{Aead, KeyInit, Payload},
        ChaCha20Poly1305,
    };
    use fastrand::Rng as FastRng;
    use ink::prelude::collections::BTreeMap;
    use scale::{Decode, Encode};
    use serde::{Deserialize, Serialize};
    use serde_json_core::*;
    use zeroize::Zeroize;

    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;

    #[derive(Decode, Encode, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // User errors
        UserExists,
        UserNotFound,

        // Password errors
        InsecurePassword,
        IncorrectMasterPassword,
        PasswordNotFound,

        // Vault errors
        VaultNotFound,

        // Crypto errors
        EncryptionError,
        DecryptionError,

        // Secret errors
        SecretNotFound,
        SecretFieldNotFound,
        InvalidSecretType,

        //Serde errors
        FailedToSerialize,
        FailedToDeserialize,
    }

    // Useful type aliases.
    pub type Result<T> = core::result::Result<T, Error>;
    pub type ChaCha20Poly1305Key = [u8; 32];
    pub type ChaCha20Poly1305Nonce = [u8; 12];
    pub type Argon2idNonce = [u8; 16];
    pub type Argon2idKey = [u8; 32];
    pub type OptionString = Option<String>;
    pub type VecTuple = Vec<(String, String)>;

    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct Credential {
        username: String,
        password: String,
    }

    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct CustomFields {
        fields: VecTuple,
    }

    #[derive(Decode, Encode, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub enum Secret {
        SoftwareLicense {
            name: String,
            key: String,
            validity: String,
            provider: OptionString,
            url: OptionString,
            #[serde(flatten)]
            other_info: CustomFields,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        WifiPassword {
            network_name: String,
            password: String,
            #[serde(flatten)]
            other_info: CustomFields,
            created_at: String,
            updated_at: String,
        },

        SSHKey {
            username: OptionString,
            email: OptionString,
            pub_key: String,
            private_key: String,
            #[serde(flatten)]
            others: CustomFields,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        Card {
            card_name: String,
            cardholder_name: String,
            card_no: String,
            security_code: String,
            expiration_month: String,
            expiration_year: String,
            pin: OptionString,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        BankAccount {
            bank_name: String,
            acc_holder: String,
            acc_no: String,
            branch: String,
            login: Option<Credential>,
            other_info: CustomFields,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        DriverLicense {
            id: String,
            category: String,
            id_type: String,
            date_of_issue: String,
            valid_till: String,
            other_info: OptionString,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        ID {
            name: String,
            id_no: String,
            date_of_birth: String,
            age: String,
            address: OptionString,
            #[serde(flatten)]
            other_info: CustomFields,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        SecureNote {
            title: OptionString,
            note: String,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        Mail {
            id: String,
            password: String,
            notes: OptionString,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        CryptoWallet {
            seed_phrase: String,
            private_key: String,
            pub_key: String,
            #[serde(flatten)]
            other_info: CustomFields,
            created_at: String,
            updated_at: String,
        },

        Login {
            username: OptionString,
            password: String,
            url: OptionString,
            notes: OptionString,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },

        Contact {
            name: String,
            number: String,
            email: String,
            tags: Vec<OptionString>,
            created_at: String,
            updated_at: String,
        },
    }

    impl Secret {
        pub fn new(name: &str) -> Result<Self> {
            match name {
                "login" => Ok(Self::Login {
                    username: None,
                    password: String::new(),
                    url: None,
                    notes: None,
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "contact" => Ok(Self::Contact {
                    name: String::new(),
                    number: String::new(),
                    email: String::new(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "mail" => Ok(Self::Mail {
                    id: String::new(),
                    password: String::new(),
                    notes: None,
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "crypto_wallet" => Ok(Self::CryptoWallet {
                    seed_phrase: String::new(),
                    private_key: String::new(),
                    pub_key: String::new(),
                    other_info: CustomFields::default(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "ID" => Ok(Self::ID {
                    name: String::new(),
                    id_no: String::new(),
                    date_of_birth: String::new(),
                    age: String::new(),
                    address: None,
                    other_info: CustomFields::default(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "card" => Ok(Self::Card {
                    card_name: String::new(),
                    cardholder_name: String::new(),
                    card_no: String::new(),
                    security_code: String::new(),
                    expiration_month: String::new(),
                    expiration_year: String::new(),
                    pin: None,
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "bank_account" => Ok(Self::BankAccount {
                    bank_name: String::new(),
                    acc_holder: String::new(),
                    acc_no: String::new(),
                    branch: String::new(),
                    login: None,
                    other_info: CustomFields::default(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "secure_note" => Ok(Self::SecureNote {
                    title: None,
                    note: String::new(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "driver_license" => Ok(Self::DriverLicense {
                    id: String::new(),
                    category: String::new(),
                    id_type: String::new(),
                    date_of_issue: String::new(),
                    valid_till: String::new(),
                    other_info: None,
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "software_license" => Ok(Self::SoftwareLicense {
                    name: String::new(),
                    key: String::new(),
                    validity: String::new(),
                    provider: None,
                    url: None,
                    other_info: CustomFields::default(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "ssh" => Ok(Self::SSHKey {
                    username: None,
                    email: None,
                    pub_key: String::new(),
                    private_key: String::new(),
                    others: CustomFields::default(),
                    tags: Vec::new(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                "wifi" => Ok(Self::WifiPassword {
                    network_name: String::new(),
                    password: String::new(),
                    other_info: CustomFields::default(),
                    created_at: String::new(),
                    updated_at: String::new(),
                }),
                // input should match with any one of the above cases. should not reach here.
                _ => Err(Error::InvalidSecretType),
            }
        }
    }

    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct Vault {
        vault: BTreeMap<String, Secret>,
    }

    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct VaultList {
        vaults: BTreeMap<String, Vault>,
    }

    pub enum UpdateVault {
        ADD,
        MODIFY,
    }

    // Vault encrypted
    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct EncryptedVaultInfo {
        key: Vec<u8>,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
    }

    // Vault list encrypted
    #[derive(Decode, Encode, Default, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
    pub struct EncryptedVaultInfoList {
        encrypted_vaults: BTreeMap<String, EncryptedVaultInfo>,
    }

    // Vec<AccountId> encrypted
    // type EncryptedAccessInfo = EncryptedVaultInfo;

    #[ink(storage)]
    #[derive(Default)]
    pub struct PassManager {
        vaults: BTreeMap<AccountId, EncryptedVaultInfoList>,
        // access_list: BTreeMap<AccountId, EncryptedAccessInfo>,
        random: u64, // safe to have because it is encrypted
    }

    impl PassManager {
        #[ink(constructor)]
        pub fn new() -> Self {
            let entropy1 = Self::env().block_timestamp();
            let entropy2 = Self::env().block_number();
            let seed = (entropy1 ^ entropy2 as u64) | (entropy1 & entropy2 as u64);
            let mut rng = FastRng::with_seed(seed);
            let randomized = rng.u64(..);

            Self {
                vaults: BTreeMap::new(),
                random: randomized,
            }
        }

        #[ink(message, payable)]
        pub fn add_user(&mut self, master_password: String) -> Result<()> {
            let caller = self.env().caller();
            self.update_seed();

            if !is_password_strong(master_password.clone()) {
                return Err(Error::InsecurePassword);
            }
            let (derive_enc_key, nonce) = self.derive_encryption_key(master_password);
            let encrypted_info = EncryptedVaultInfo {
                key: derive_enc_key.to_vec(),
                ciphertext: Vec::new(),
                nonce: nonce.to_vec(),
            };

            let mut encrypted_vaults = BTreeMap::new();
            encrypted_vaults.insert("Personal Vault".to_string(), encrypted_info);
            let encrypted_list = EncryptedVaultInfoList { encrypted_vaults };
            self.vaults.insert(caller, encrypted_list);
            return Ok(());
            
        }

        #[ink(message, payable)]
        pub fn add_secret(
            &mut self,
            vault_name: String,
            secret_name: String,
            fields: VecTuple,
            secret_type: String,
        ) -> Result<()> {
            let caller = self.env().caller();
            if !self.vaults.contains_key(&caller) {
                return Err(Error::UserNotFound);
            } else {
                let (_, mut get_vaults) = self.vaults.remove_entry(&caller).ok_or(Error::UserNotFound)?;
                let get_vault = get_vaults.encrypted_vaults.get_mut(&vault_name).ok_or(Error::VaultNotFound)?;
                let mut decrypted = self.decrypt_vault(get_vault.clone())?;
                let mut secret = Secret::new(&secret_type)?;
                Self::update_vault(&mut secret, fields, UpdateVault::ADD)?;

                decrypted.vault.insert(secret_name, secret);
                *get_vault = self.encrypt_vault(decrypted)?;

                self.vaults.insert(caller, get_vaults);
            };
            return Ok(());
        }

        #[ink(message, payable)]
        pub fn modify_secret(
            &mut self,
            vault_name: String,
            secret_name: String,
            fields: VecTuple,
        ) -> Result<()> {
            let caller = self.env().caller();
            if self.is_user(caller) {
                let (_, mut get_vaults) = self
                    .vaults
                    .remove_entry(&caller)
                    .ok_or(Error::VaultNotFound)?;
                let get_vault = get_vaults
                    .encrypted_vaults
                    .get_mut(&vault_name)
                    .ok_or(Error::VaultNotFound)?;
                let mut decrypted = self.decrypt_vault(get_vault.clone())?;

                let secret = decrypted
                    .vault
                    .get_mut(&secret_name)
                    .ok_or(Error::SecretNotFound)?;

                let secret_type = match secret.clone() {
                    Secret::Login { .. } => "login",
                    Secret::WifiPassword { .. } => "wifi",
                    Secret::DriverLicense { .. } => "driver_license",
                    Secret::SoftwareLicense { .. } => "software_license",
                    Secret::Card { .. } => "card",
                    Secret::BankAccount { .. } => "bank_account",
                    Secret::CryptoWallet { .. } => "crypto_wallet",
                    Secret::Mail { .. } => "mail",
                    Secret::SecureNote { .. } => "secure_note",
                    Secret::SSHKey { .. } => "ssh",
                    Secret::ID { .. } => "ID",
                    Secret::Contact { .. } => "contact",
                };

                let mut new_instance = Secret::new(secret_type)?;

                Self::update_vault(&mut new_instance, fields, UpdateVault::MODIFY)?;

                *secret = new_instance;

                let encrypted = self.encrypt_vault(decrypted)?;

                *get_vault = encrypted;

                self.vaults.insert(caller, get_vaults);

                return Ok(());
            } else {
                return Err(Error::UserNotFound);
            }
        }

        #[ink(message, payable)]
        pub fn delete_secret(&mut self, secret_name: String, vault_name: String) -> Result<()> {
            let caller = self.env().caller();
            if !self.is_user(caller) {
                return Err(Error::UserNotFound);
            } else {
                let mut get_vaults = self
                    .vaults
                    .remove_entry(&caller)
                    .ok_or(Error::UserNotFound)?;
                let get_vault = get_vaults
                    .1
                    .encrypted_vaults
                    .get_mut(&vault_name)
                    .ok_or(Error::VaultNotFound)?;
                let mut decrypted = self.decrypt_vault(get_vault.clone())?;

                let remove_secret = decrypted.vault.remove_entry(&secret_name);

                match remove_secret {
                    Some(_) => {
                        let encrypt = self.encrypt_vault(decrypted)?;

                        *get_vault = encrypt;

                        self.vaults.insert(get_vaults.0, get_vaults.1);
                        return Ok(());
                    }
                    None => {
                        let encrypt = self.encrypt_vault(decrypted)?;

                        *get_vault = encrypt;

                        self.vaults.insert(get_vaults.0, get_vaults.1);
                        return Err(Error::SecretNotFound);
                    }
                }
            }
        }

        fn update_vault(
            secret: &mut Secret,
            fields: VecTuple,
            mutate_type: UpdateVault,
        ) -> Result<Secret> {
            let secret_cloned = secret.clone();
            match secret_cloned {
                Secret::Login { .. } => {
                    if let Secret::Login {
                        username,
                        password,
                        url,
                        notes,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "username" => *username = Some(value),
                                "password" => {
                                    if !is_password_strong(value.to_owned()) {
                                        return Err(Error::InsecurePassword);
                                    } else {
                                        *password = value;
                                    }
                                }
                                "url" => *url = Some(value),
                                "notes" => *notes = Some(value),
                                "tags" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::Contact { .. } => {
                    if let Secret::Contact {
                        name,
                        number,
                        email,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "name" => *name = value,
                                "number" => *number = value,
                                "email" => *email = value,
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::Mail { .. } => {
                    if let Secret::Mail {
                        id,
                        password,
                        notes,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "id" => *id = value,
                                "password" => {
                                    if !is_password_strong(field_name.to_owned()) {
                                        return Err(Error::InsecurePassword);
                                    } else {
                                        *password = value;
                                    }
                                }
                                "notes" => *notes = Some(value),
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::CryptoWallet { .. } => {
                    if let Secret::CryptoWallet {
                        seed_phrase,
                        private_key,
                        pub_key,
                        other_info,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "seed_phrase" => *seed_phrase = value,
                                "private_key" => *private_key = value,
                                "pub_key" => *pub_key = value,
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        other_info.fields.push(sub_secret);
                                    }
                                }
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::SecureNote { .. } => {
                    if let Secret::SecureNote {
                        title,
                        note,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "title" => *title = Some(value),
                                "note" => *note = value,
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::ID { .. } => {
                    if let Secret::ID {
                        name,
                        id_no,
                        date_of_birth,
                        age,
                        address,
                        other_info,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "name" => *name = value,
                                "id" => *id_no = value,
                                "dob" => *date_of_birth = value,
                                "age" => *age = value,
                                "address" => *address = Some(value),
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        other_info.fields.push(sub_secret);
                                    }
                                }
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::Card { .. } => {
                    if let Secret::Card {
                        card_name,
                        cardholder_name,
                        card_no,
                        security_code,
                        expiration_month,
                        expiration_year,
                        pin,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "cardname" => *card_name = value,
                                "cardholder" => *cardholder_name = value,
                                "card_no" => *card_no = value,
                                "security_code" => *security_code = value,
                                "expiration_month" => *expiration_month = value,
                                "expiration_year" => *expiration_year = value,
                                "pin" => *pin = Some(value),
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::BankAccount { .. } => {
                    if let Secret::BankAccount {
                        bank_name,
                        acc_holder,
                        acc_no,
                        branch,
                        login,
                        other_info,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "bank_name" => *bank_name = value,
                                "acc_holder" => *acc_holder = value,
                                "acc_no" => *acc_no = value,
                                "branch" => *branch = value,
                                "login" => {
                                    let deserialized = from_str::<Credential>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    *login = Some(deserialized.0);
                                }
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        other_info.fields.push(sub_secret);
                                    }
                                }
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::DriverLicense { .. } => {
                    if let Secret::DriverLicense {
                        id,
                        category,
                        id_type,
                        date_of_issue,
                        valid_till,
                        other_info,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "id" => *id = value,
                                "category" => *category = value,
                                "id_type" => *id_type = value,
                                "date_of_issue" => *date_of_issue = value,
                                "validity" => *valid_till = value,
                                "other_info" => *other_info = Some(value),
                                "tag" => tags.push(Some(value)),
                                "updated_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::SoftwareLicense { .. } => {
                    if let Secret::SoftwareLicense {
                        name,
                        key,
                        validity,
                        provider,
                        url,
                        other_info,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "name" => *name = value,
                                "key" => *key = value,
                                "validity" => *validity = value,
                                "provider" => *provider = Some(value),
                                "url" => *url = Some(value),
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        other_info.fields.push(sub_secret);
                                    }
                                }
                                "tag" => tags.push(Some(value)),
                                "updated_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::WifiPassword { .. } => {
                    if let Secret::WifiPassword {
                        network_name,
                        password,
                        other_info,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "network" => *network_name = value,
                                "password" => *password = value,
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        other_info.fields.push(sub_secret);
                                    }
                                }
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Secret::SSHKey { .. } => {
                    if let Secret::SSHKey {
                        username,
                        email,
                        pub_key,
                        private_key,
                        others,
                        tags,
                        created_at,
                        updated_at,
                    } = secret
                    {
                        for (field_name, value) in fields.into_iter() {
                            match field_name.as_str() {
                                "username" => *username = Some(value),
                                "email" => *email = Some(value),
                                "pub_key" => *pub_key = value,
                                "private_key" => *private_key = value,
                                "other_info" => {
                                    let deserialized = from_str::<CustomFields>(&value)
                                        .map_err(|_| Error::FailedToDeserialize)?;
                                    for sub_secret in deserialized.0.fields.into_iter() {
                                        others.fields.push(sub_secret);
                                    }
                                }
                                "tag" => tags.push(Some(value)),
                                "created_at" => match mutate_type {
                                    UpdateVault::ADD => {
                                        *created_at = value.clone();
                                        *updated_at = value;
                                    }
                                    UpdateVault::MODIFY => {
                                        *updated_at = value;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
            return Ok(secret.clone());
        }

        pub fn encrypt_vault(&mut self, vault: Vault) -> Result<EncryptedVaultInfo> {
            let key = self.gen_key();
            let nonce = self.gen_nonce();

            let data = bin_serde::to_allocvec(&vault).unwrap();

            let payload = Payload {
                msg: &data,
                aad: b"", // AD to be added later
            };

            let chapoly = ChaCha20Poly1305::new(&key.into());
            let ciphertext = chapoly
                .encrypt(&nonce.into(), payload)
                .map_err(|_| Error::EncryptionError)?;
            self.update_seed();

            Ok(EncryptedVaultInfo {
                key: key.to_vec(),
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }

        pub fn decrypt_vault(&mut self, encryption: EncryptedVaultInfo) -> Result<Vault> {
            let key = encryption.key.clone();
            let nonce = encryption.nonce.clone();

            let mut key_fixed = [0u8; 32];
            let mut nonce_fixed = [0u8; 12];

            for i in 0..key_fixed.len() {
                key_fixed[i] = key[i];
            }

            for i in 0..nonce_fixed.len() {
                nonce_fixed[i] = nonce[i];
            }
            let chapoly = ChaCha20Poly1305::new(&key_fixed.into());
            let vault = chapoly
                .decrypt(&nonce_fixed.into(), &encryption.ciphertext[..])
                .map_err(|_| Error::DecryptionError)?;
            let decrypted = bin_serde::from_bytes::<'_, Vault>(&vault)
                .map_err(|_| Error::FailedToDeserialize)?;
            Ok(decrypted)
        }

        fn derive_encryption_key(&mut self, mut pass: String) -> (Argon2idKey, Argon2idNonce) {
            let mut rng = FastRng::with_seed(self.random);

            let mut encryption_key = [0; 32];
            let mut nonce = [0; 16];
            rng.fill(&mut nonce);

            // Params from https://en.wikipedia.org/wiki/Argon2
            let argon_params = Params::new(46, 1, 1, None).unwrap();
            let argon2id = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

            argon2id
                .hash_password_into(pass.as_bytes(), &nonce[..], &mut encryption_key)
                .unwrap();
            pass.zeroize();

            (encryption_key, nonce)
        }

        fn update_seed(&mut self) {
            let mut rng = FastRng::with_seed(self.random);
            let mut rand = [0u8; 32];
            rng.fill(&mut rand);
            let randomized = bytes_to_num(&rand);
            self.random = randomized;
        }

        fn gen_key(&mut self) -> ChaCha20Poly1305Key {
            let mut rng = FastRng::with_seed(self.random);
            let mut rand = [0u8; 32];
            rng.fill(&mut rand);
            self.update_seed();

            rand
        }

        fn gen_nonce(&mut self) -> ChaCha20Poly1305Nonce {
            let mut rng = FastRng::with_seed(self.random);
            let mut rand = [0u8; 12];
            rng.fill(&mut rand);
            self.update_seed();

            rand
        }

        pub fn is_user(&self, caller: AccountId) -> bool {
            if self.vaults.contains_key(&caller) {
                true
            } else {
                false
            }
        }
    }
}