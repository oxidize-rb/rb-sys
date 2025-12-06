use anyhow::{Context, Result};
use oci_distribution::client::{ClientConfig, ClientProtocol};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        anyhow::bail!("Usage: {} <image-tag>", args[0]);
    }
    
    let image_tag = &args[1];
    let digest = fetch_image_digest(image_tag).await?;
    
    println!("{}", digest);
    
    Ok(())
}

async fn fetch_image_digest(image_ref: &str) -> Result<String> {
    let reference: Reference = image_ref
        .parse()
        .with_context(|| format!("Failed to parse image reference: {}", image_ref))?;

    let config = ClientConfig {
        protocol: ClientProtocol::Https,
        ..Default::default()
    };
    
    let client = Client::new(config);
    let auth = get_registry_auth();

    // Pull manifest to get digest
    let (_, digest) = client
        .pull_manifest(&reference, &auth)
        .await
        .with_context(|| format!("Failed to fetch manifest for: {}", image_ref))?;

    Ok(digest)
}

fn get_registry_auth() -> RegistryAuth {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        RegistryAuth::Basic("token".to_string(), token)
    } else {
        RegistryAuth::Anonymous
    }
}
