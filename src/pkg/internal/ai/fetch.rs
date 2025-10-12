use reqwest::Url;
use scraper::{Html, Selector};

pub async fn process(url: &str) -> String {
    if Url::parse(url).is_err() {
        return url.into();
    }
    match fetch_and_extract(&url).await {
        Ok(text) => text,
        Err(_) => url.into(),
    }
}

async fn fetch_and_extract(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    tracing::debug!("Fetching URL: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;
    let status = response.status();
    tracing::debug!("Response status: {}", status);
    if status.as_u16() == 999 {
        tracing::warn!("Received 999 status - likely bot detection");
        return Err("Bot detection - 999 status code".into());
    }
    let html_content = response.text().await?;
    tracing::debug!("HTML content length: {} bytes", html_content.len());
    tracing::debug!(
        "First 200 chars of HTML: {}",
        &html_content.chars().take(200).collect::<String>()
    );
    let document = Html::parse_document(&html_content);
    tracing::debug!("HTML parsed successfully");
    let text_selector = Selector::parse("body").unwrap();
    let mut text_parts = Vec::new();
    let body_count = document.select(&text_selector).count();
    tracing::debug!("Found {} body elements", body_count);
    for (idx, element) in document.select(&text_selector).enumerate() {
        tracing::debug!("Processing body element {}", idx);
        let html = element.html();
        tracing::debug!("Body HTML length: {} bytes", html.len());
        let text = element
            .text()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        tracing::debug!("Extracted text length: {} chars", text.len());
        tracing::debug!(
            "First 100 chars: {}",
            &text.chars().take(100).collect::<String>()
        );
        if !text.is_empty() {
            text_parts.push(text);
        }
    }
    // Fallback: if body is empty, try getting all text from the document
    if text_parts.is_empty() {
        tracing::debug!("Body was empty, trying to extract all text from document");
        let all_text: String = document
            .root_element()
            .text()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        tracing::debug!("Fallback extracted text length: {} chars", all_text.len());

        if !all_text.is_empty() {
            text_parts.push(all_text);
        }
    }

    tracing::debug!("Total text parts: {}", text_parts.len());
    let result = text_parts.join("\n");
    tracing::debug!("Final result length: {} chars", result.len());

    Ok(result)
}
