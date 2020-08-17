mod api;
mod email;

use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone};
use clokwerk::{Scheduler, TimeUnits};
use math::round;

use api::{listing_details, ListingDetails};
use email::send_message;
use tokio::{spawn, time};
use unindent::unindent;

use std::env;
extern crate base64;
use rust_embed::RustEmbed;

#[macro_use]
extern crate lazy_static;

const DEPOSIT: i16 = 3075;
const RENT_PM: i16 = 2665;
const TENANCY_DURATION: i16 = 365;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

lazy_static! {
    static ref END_DATE: DateTime<FixedOffset> =
        DateTime::parse_from_str("2020 Aug 12 00:00:00 +0100", "%Y %b %d %H:%M:%S %z").unwrap();
}

fn calc_duration<Tz: TimeZone>(since: DateTime<Tz>) -> Duration {
    since.signed_duration_since(*END_DATE)
}

fn create_message(listing: ListingDetails, days_lost: i16) -> String {
    let rent_pd = round::half_up((RENT_PM * 12) as f64 / 365.0, 2);

    unindent(
        &format!(
            "
            From: HELLO BOYS ITS PEPPA <helloboysitspeppa@gmail.com>
            Subject: {} day(s) of no one living in 52 Castletown road
            Content-Type: text/html; charset='utf-8'
            Content-Transfer-Encoding: quoted-printable
            Content-Disposition: inline

            <!DOCTYPE HTML PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN'>
            <html>
                <head>

                <meta http-equiv='content-type' content='text/html; charset=ISO-8859-15'>
                </head>
                <body>
                    <p>Dear all,</p>

                    <p>52 Castletown road is still on the market, meaning so far as least £{} of lost income for Peppa.</p>

                    <p>- {}</p>
                    <p>- {}</p>

                    <p>The property is currently being listed at £{} PM</p>
                    
                    <p>Enjoy your day!</p>
                    <img src='data:image/png;base64,{}' alt='pepppppaaaaaa'>
                </body>
            </html>
            ",
            days_lost,
            round::floor(rent_pd * days_lost as f64, 0),
            match round::floor(DEPOSIT as f64 / rent_pd, 0) as i16 {
                deposit_days if deposit_days - days_lost <= 0 => "the deposit now doesn't cover lost rent".to_string(),
                deposit_days if deposit_days - days_lost > 0 => format!(
                    "{} days until the deposit doesn't cover the lost rent ({}% of the way)", 
                    deposit_days - days_lost,
                    ((days_lost as f32 / deposit_days as f32) * 100 as f32) as i8
                ),
                _ => "Not sure how this happened".to_string()
            },
            match TENANCY_DURATION - days_lost <= 0 {
                true => "our rent now doesn't cover loses".to_string(),
                false => format!(
                    "{} days until our tenancy doesn't cover the lost rent ({}% of the way)", 
                    TENANCY_DURATION - days_lost,
                    ((days_lost as f32 / TENANCY_DURATION as f32) * 100 as f32) as i8
                ),
            },
            listing.price_pm,
            base64::encode(Asset::get("peppa.png").unwrap().as_ref())
        )
    )
}

async fn notify_updates(
    recips: Vec<String>,
    password: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = "https://www.rightmove.co.uk/property-to-rent/property-80785048.html";
    let listing = listing_details(address).await?;
    if !listing.has_been_let {
        send_message(
            recips,
            create_message(listing, calc_duration(Local::now()).num_days() as i16),
            password,
        )
        .await?
    };

    Ok(())
}

#[tokio::main]
async fn main() -> () {
    let recips = env::var("EMAILS")
        .unwrap_or("".into())
        .split_whitespace()
        .collect::<String>()
        .split(",")
        .map(|r| r.to_string())
        .filter(|r| r.len() > 0 && r.contains("@"))
        .collect::<Vec<String>>();

    if recips.len() < 1 {
        println!("EMAILS is a required environment variable");
        return;
    }

    let password = env::var("SMTP_PASSWORD").unwrap_or("".into());

    if password.len() < 1 {
        println!("SMTP_PASSWORD is required");
        return;
    }

    println!("Scheduling notifications every day at 17:00");
    println!("Using recipients {}", recips.join(", "));

    let mut scheduler = Scheduler::new();
    scheduler.every(1.days()).at("17:00").run(move || {
        let recips = recips.clone();
        let password = password.clone();
        spawn(async move {
            match notify_updates(recips, password).await {
                Ok(_) => println!("Successfully notified"),
                Err(e) => println!("could not notify: {}", e),
            }
        });
        ()
    });

    println!("Polling scheduler");
    loop {
        scheduler.run_pending();
        time::delay_for(std::time::Duration::from_millis(100)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::ListingDetails;
    use insta::assert_snapshot;
    #[test]
    fn test_message_one_day() {
        let message = create_message(
            ListingDetails {
                price_pm: 2665,
                has_been_let: true,
            },
            1,
        );

        assert_snapshot!(message);
    }

    #[test]
    fn test_message_30_days() {
        let message = create_message(
            ListingDetails {
                price_pm: 2665,
                has_been_let: true,
            },
            30,
        );

        assert_snapshot!(message);
    }

    #[test]
    fn test_message_deposit_lost() {
        let message = create_message(
            ListingDetails {
                price_pm: 2665,
                has_been_let: true,
            },
            50,
        );

        assert_snapshot!(message);
    }
    #[test]

    fn test_message_rent_lost() {
        let message = create_message(
            ListingDetails {
                price_pm: 2665,
                has_been_let: true,
            },
            365,
        );

        assert_snapshot!(message);
    }

    #[test]
    fn test_calc_duration() {
        assert_eq!(
            calc_duration(
                DateTime::parse_from_str("2020 Aug 12 00:00:00 +0100", "%Y %b %d %H:%M:%S %z")
                    .unwrap()
            )
            .num_days(),
            0
        );
        assert_eq!(
            calc_duration(
                DateTime::parse_from_str("2020 Aug 20 00:00:00 +0100", "%Y %b %d %H:%M:%S %z")
                    .unwrap()
            )
            .num_days(),
            8
        );
        assert_eq!(
            calc_duration(
                DateTime::parse_from_str("2020 Sep 12 00:00:00 +0100", "%Y %b %d %H:%M:%S %z")
                    .unwrap()
            )
            .num_days(),
            31
        );
    }
}
