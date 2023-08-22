# Hello World in Phat Contract

<img align="right" width="320" src="res/Phat%20Contract_Standard%20Logo_wht_02.svg">

This contract shows the off-chain computation with native HTTP request support in Phat Contract from [Phala Network](https://phala.network/).

It receives the Ethereum accounts from users and reports the account balance by querying the Etherscan with its native HTTP request.

## Build

Setup the environment for Ink! contract compilation following the [official tutorial](https://github.com/paritytech/cargo-contract#installation)

```bash
# tested on Ubuntu 22.04
cargo contract --version
# cargo-contract-contract 3.0.1-unknown-x86_64-unknown-linux-gnu
```

then run

```bash
cargo contract build
```

## Test

To test your contract locally and see its output, run with

```bash
cargo test -- --nocapture
```

https://api.openai.com/v1/images/generations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer "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"" \
  -d '{
    "prompt": "a white siamese cat",
    "n": 1,
    "size": "256x256"
  }'

  #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Authorization", "Bearer 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".parse()?);

    let data = r#"{
    "prompt": "a white siamese cat",
    "n": 1,
    "size": "256x256"
}"#;

    let json: serde_json::Value = serde_json::from_str(&data)?;

    let request = client.request(reqwest::Method::POST, "https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(&json);

    let response = request.send().await?;
    let body = response.text().await?;

    println!("{}", body);

    Ok(())
}