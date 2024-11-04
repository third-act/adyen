use rust_decimal::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
mod error;
pub use error::Error;
mod currency;
pub use currency::Currency;
mod card_on_file;
pub use card_on_file::CardOnFile;
mod pay_with_card_on_file;
mod pay_with_new_card_on_file;
pub mod prelude {
    pub use super::{CardOnFile, Currency, Gateway};
}

pub struct Gateway {
    client: reqwest::Client,
    _api_key: String,
}

pub fn convert_decimal_into_minor_units<'a>(
    amount: &'a Decimal,
    currency: &'a Currency,
) -> Result<u64, Error> {
    let decimals = match currency {
        Currency::NOK => 2,
        Currency::SEK => 2,
        Currency::DKK => 2,
        Currency::ISK => 0,
        Currency::GBP => 2,
        Currency::EUR => 2,
    };

    // Get minor units from decimal.
    let minor_units_modifier = Decimal::from(10i32.pow(decimals));
    let amount = amount * minor_units_modifier;

    match amount.to_u64() {
        Some(a) => Ok(a),
        None => {
            return Err(Error::ConversionError(format!(
                "could not convert \"{}\" to u64",
                amount
            )))
        }
    }
}

impl Gateway {
    pub fn new(api_key: String, timeout: Option<Duration>) -> Result<Gateway, Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let x_api_key_header_value = format!("{}", api_key);
        let x_api_key_header = match reqwest::header::HeaderValue::from_str(&x_api_key_header_value)
        {
            Ok(header) => header,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "could not create auth header ({})",
                    err.to_string()
                )))
            }
        };

        headers.insert("x-API-key", x_api_key_header);

        let timeout = match timeout {
            Some(t) => t,
            None => Duration::new(60, 0),
        };

        let client = match reqwest::ClientBuilder::new()
            .default_headers(headers)
            .https_only(true)
            .timeout(timeout)
            .build()
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "could not create reqwest client ({})",
                    err.to_string()
                )))
            }
        };

        Ok(Gateway {
            client,
            _api_key: api_key,
        })
    }

    async fn post<'a, T: DeserializeOwned>(
        &self,
        url: &str,
        body: impl Serialize,
    ) -> Result<T, Error> {
        let res = match self.client.post(url).json(&body).send().await {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::NetworkError(format!(
                    "could not send request ({})",
                    err.to_string()
                )))
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        if status.as_u16() < 200 || status.as_u16() >= 300 {
            // #[derive(Deserialize, Debug, Clone)]
            // #[serde(rename_all = "camelCase")]
            // struct ApiError {
            //     pub meta: Option<Meta>,
            // }

            // let api_error: ApiError =
            //     serde_json::from_str(&text).unwrap_or_else(|_| ApiError { meta: None });

            // let meta = match api_error.meta {
            //     Some(m) => m,
            //     None => {
            //         return Err(Error::ApiError(
            //             status.as_u16().to_string(),
            //             "unknown".to_string(),
            //             None,
            //             None,
            //             text,
            //         ))
            //     }
            // };

            // let action = match meta.action {
            //     Some(m) => m,
            //     None => {
            //         return Err(Error::ApiError(
            //             status.as_u16().to_string(),
            //             "unknown".to_string(),
            //             None,
            //             None,
            //             text,
            //         ))
            //     }
            // };

            // let code = match action.code {
            //     Some(m) => m,
            //     None => {
            //         return Err(Error::ApiError(
            //             status.as_u16().to_string(),
            //             "unknown".to_string(),
            //             None,
            //             None,
            //             text,
            //         ))
            //     }
            // };

            // let source = match action.source {
            //     Some(m) => m,
            //     None => {
            //         return Err(Error::ApiError(
            //             status.as_u16().to_string(),
            //             "unknown".to_string(),
            //             None,
            //             None,
            //             text,
            //         ))
            //     }
            // };

            // let (enduser_message, merchant_message) = match meta.message {
            //     Some(m) => (m.enduser, m.merchant),
            //     None => (None, None),
            // };

            // return Err(Error::ApiError(
            //     code,
            //     source,
            //     enduser_message,
            //     merchant_message,
            //     text,
            // ));

            todo!()
        }

        let body: T = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::SerializationError(format!(
                    "could not deserialize response ({}): {}",
                    err, text
                )))
            }
        };
        Ok(body)
    }

    // async fn get<'a, T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
    //     let res = match self.client.get(url).send().await {
    //         Ok(r) => r,
    //         Err(err) => {
    //             return Err(Error::NetworkError(format!(
    //                 "could not send request ({})",
    //                 err
    //             )))
    //         }
    //     };

    //     let status = res.status();
    //     let text = res
    //         .text()
    //         .await
    //         .unwrap_or_else(|_| String::from("Could not retrieve body text."));

    //     if status.as_u16() < 200 || status.as_u16() >= 300 {
    //         #[derive(Deserialize, Debug, Clone)]
    //         #[serde(rename_all = "camelCase")]
    //         struct ApiError {
    //             pub meta: Option<Meta>,
    //         }

    //         let api_error: ApiError =
    //             serde_json::from_str(&text).unwrap_or_else(|_| ApiError { meta: None });

    //         let meta = match api_error.meta {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let action = match meta.action {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let code = match action.code {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let source = match action.source {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let (enduser_message, merchant_message) = match meta.message {
    //             Some(m) => (m.enduser, m.merchant),
    //             None => (None, None),
    //         };

    //         return Err(Error::ApiError(
    //             code,
    //             source,
    //             enduser_message,
    //             merchant_message,
    //             text,
    //         ));
    //     }

    //     let body: T = match serde_json::from_str(&text) {
    //         Ok(r) => r,
    //         Err(err) => {
    //             return Err(Error::SerializationError(format!(
    //                 "could not deserialize response ({}): {}",
    //                 err, text
    //             )))
    //         }
    //     };
    //     Ok(body)
    // }
}
