use crate::Result;

pub async fn sort(href: &str, compact: bool) -> Result<()> {
    let value: stac::Value = stac_async::read_json(href).await?;
    let output = if compact {
        serde_json::to_string(&value).unwrap()
    } else {
        serde_json::to_string_pretty(&value).unwrap()
    };
    println!("{}", output);
    Ok(())
}
