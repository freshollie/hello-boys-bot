use async_smtp::smtp::authentication::Credentials;
use async_smtp::{EmailAddress, Envelope, SendableEmail, SmtpClient};

pub async fn send_message(
    recips: Vec<String>,
    message: String,
    password: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = SendableEmail::new(
        Envelope::new(
            Some("helloboysitspeppa@gmail.com".parse().unwrap()),
            recips
                .iter()
                .map(|r| EmailAddress::new(r.into()).unwrap())
                .collect(),
        )?,
        "id",
        message,
    );
    let mut smtp = SmtpClient::new("smtp.gmail.com")
        .await?
        .credentials(Credentials::new(
            "helloboysitspeppa@gmail.com".to_string(),
            password,
        ))
        .into_transport();

    smtp.connect_and_send(email).await?;

    Ok(())
}
