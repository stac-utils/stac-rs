use crate::Result;
use stac::Value;

pub async fn sort(href: Option<&str>, compact: bool) -> Result<()> {
    let value: Value = if let Some(href) = href {
        stac_async::read_json(href).await?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };
    let output = if compact {
        serde_json::to_string(&value).unwrap()
    } else {
        serde_json::to_string_pretty(&value).unwrap()
    };
    println!("{}", output);
    Ok(())
}
