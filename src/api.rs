use regex::Regex;
use reqwest::header::USER_AGENT;

pub struct ListingDetails {
    pub has_been_let: bool,
    pub price_pm: i32,
}

pub async fn listing_details(url: &str) -> Result<ListingDetails, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4147.105 Safari/537.36")
        .send()
        .await?
        .text()
        .await?
        .to_ascii_lowercase();

    let has_been_let = resp.contains("let agreed");

    lazy_static! {
        static ref PRICE_MATCHER: Regex = Regex::new("(?P<price>[0-9]+) pw").unwrap();
    };

    if let Some(matched) = PRICE_MATCHER.captures(&resp) {
        if let Some(price_string) = matched.name("price") {
            return Ok(ListingDetails {
                price_pm: price_string.as_str().parse::<i32>().unwrap() * 52 / 12,
                has_been_let,
            });
        }
    }
    Err("could not find price info".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::{Mock, MockServer};
    use insta::*;
    use std::path::Path;
    use tokio::fs::read_to_string;

    async fn mock_response(
        data_file: &str,
    ) -> Result<(String, Mock, MockServer), Box<dyn std::error::Error>> {
        let path = Path::new(file!())
            .parent()
            .unwrap()
            .join("__data__")
            .join(data_file);
        let content = read_to_string(path).await?;
        let server = MockServer::start_async().await;
        let response_mock = Mock::new()
            .expect_method(GET)
            .expect_path("/")
            .return_body(&content)
            .return_status(200);

        Ok((server.url("/"), response_mock, server))
    }

    #[tokio::test]
    async fn test_listing_let_agreed() {
        let (address, mock, server) = mock_response("let_agreed.html").await.unwrap();
        mock.create_on(&server);

        let listing = listing_details(&address).await.unwrap();
        assert_snapshot!(listing.price_pm.to_string());
        assert_eq!(listing.has_been_let, true);
    }

    #[tokio::test]
    async fn test_listing_let_not_agreed() {
        let (address, mock, server) = mock_response("no_let_agreed.html").await.unwrap();
        mock.create_on(&server);
        let listing = listing_details(&address).await.unwrap();

        assert_snapshot!(listing.price_pm.to_string());
        assert_eq!(listing.has_been_let, false);
    }
}
