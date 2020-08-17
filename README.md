<img align="right" src="assets/peppa.png" height="200px" style="transform: scaleX(-1);">

# hello-boys-bot

[![Pipeline status](https://github.com/freshollie/hello-boys-bot/workflows/pipeline/badge.svg)](https://github.com/freshollie/hello-boys-bot/actions)

> Greedy landlords don't deserve tenants

## About

This program automatically emails "us" every day to let us know
how long it's been since the configured property was let, and calculates
the loss the landlord has incurred from not renting.

## Why Rust?

No reason at all. I like strongly typed modern languages, and saw this project
as an opportunity to learn `rust`. So far, it's been great!

## Tests

```shell
cargo test
```

## Running

To run this program, an email password must be supplied.

```
EMAILS="my@email.com,the@otheremail.com" SMTP_PASSWORD="somepassword" cargo run
```
