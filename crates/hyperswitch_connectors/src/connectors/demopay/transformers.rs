
// use common_enums::enums;
// use common_utils::types::StringMinorUnit;
// use hyperswitch_domain_models::{
//     payment_method_data::PaymentMethodData,
//     router_data::{ConnectorAuthType, RouterData},
//     router_flow_types::refunds::{Execute, RSync},
//     router_request_types::ResponseId,
//     router_response_types::{PaymentsResponseData, RefundsResponseData},
//     types::{PaymentsAuthorizeRouterData, RefundsRouterData},
// };
// use hyperswitch_interfaces::errors;
// use masking::Secret;
// use serde::{Deserialize, Serialize};

// use crate::types::{RefundsResponseRouterData, ResponseRouterData};


// #[derive(Debug, Clone)]
// pub struct DemopayRouterData<T> {
//     pub amount: StringMinorUnit,
//     pub router_data: T,
// }

// impl<T> From<(StringMinorUnit, T)> for DemopayRouterData<T> {
//     fn from((amount, item): (StringMinorUnit, T)) -> Self {
//         Self {
//             amount,
//             router_data: item,
//         }
//     }
// }

// #[derive(Debug, Serialize, PartialEq)]
// pub struct DemopayPaymentsRequest {
//     pub amount: StringMinorUnit,
//     pub currency: String,
//     pub wallet_id: String,
// }

// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct DemopayPaymentsResponse {
//     pub txn_id: String,
//     pub status: String, // e.g., "authorized", "captured", "failed"
//     pub message: Option<String>,
//     pub amount: Option<StringMinorUnit>,
//     pub currency: Option<String>,
// }

// impl TryFrom<&DemopayRouterData<&PaymentsAuthorizeRouterData>> for DemopayPaymentsRequest {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(
//         item: &DemopayRouterData<&PaymentsAuthorizeRouterData>,
//     ) -> Result<Self, Self::Error> {
//         let (wallet_id, currency) = match &item.router_data.request.payment_method_data {
//             PaymentMethodData::Wallet(wallet_data) => {
//                 // Example: extract wallet_id and currency from wallet_data
//                 // (Replace with actual extraction logic as per your WalletData structure)
//                 ("demo_wallet_id".to_string(), item.router_data.request.currency.to_string())
//             },
//             _ => return Err(errors::ConnectorError::NotImplemented(
//                 "Only wallet payments supported for DemoPay".to_string(),
//             )
//             .into()),
//         };
//         Ok(Self {
//             amount: item.amount.clone(),
//             currency,
//             wallet_id,
//         })
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct DemopayErrorResponse {
//     pub code: String,
//     pub message: String,
//     pub reason: Option<String>,
// }

// // Auth Struct
// pub struct DemopayAuthType {
//     pub(super) api_key: Secret<String>,
// }

// impl TryFrom<&ConnectorAuthType> for DemopayAuthType {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
//         match auth_type {
//             ConnectorAuthType::HeaderKey { api_key } => Ok(Self {
//                 api_key: api_key.to_owned(),
//             }),
//             _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
//         }
//     }
// }
// // PaymentsResponse
// #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
// #[serde(rename_all = "lowercase")]
// pub enum DemopayPaymentStatus {
//     Authorized,
//     Captured,
//     Failed,
//     #[default]
//     Processing,
// }

// impl From<DemopayPaymentStatus> for common_enums::AttemptStatus {
//     fn from(item: DemopayPaymentStatus) -> Self {
//         match item {
//             DemopayPaymentStatus::Authorized => Self::Authorized,
//             DemopayPaymentStatus::Captured => Self::Charged,
//             DemopayPaymentStatus::Failed => Self::Failure,
//             DemopayPaymentStatus::Processing => Self::Pending,
//         }
//     }
// }

// impl<F, T> TryFrom<ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>>
//     for RouterData<F, T, PaymentsResponseData>
// {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(
//         item: ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>,
//     ) -> Result<Self, Self::Error> {
//         // Map DemoPay status string to AttemptStatus
//         let status = match item.response.status.as_str() {
//             "authorized" => common_enums::AttemptStatus::Authorized,
//             "captured" => common_enums::AttemptStatus::Charged,
//             "failed" => common_enums::AttemptStatus::Failure,
//             _ => common_enums::AttemptStatus::Pending,
//         };
//         Ok(Self {
//             status,
//             response: Ok(PaymentsResponseData::TransactionResponse {
//                 resource_id: ResponseId::ConnectorTransactionId(item.response.txn_id.clone()),
//                 redirection_data: Box::new(None),
//                 mandate_reference: Box::new(None),
//                 connector_metadata: None,
//                 network_txn_id: None,
//                 connector_response_reference_id: None,
//                 incremental_authorization_allowed: None,
//                 charges: None,
//             }),
//             ..item.data
//         })
//     }
// }

// #[derive(Default, Debug, Serialize)]
// pub struct DemopayRefundRequest {
//     pub amount: StringMinorUnit,
//     pub currency: String,
//     pub payment_id: String,
// }

// impl<F> TryFrom<&DemopayRouterData<&RefundsRouterData<F>>> for DemopayRefundRequest {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(item: &DemopayRouterData<&RefundsRouterData<F>>) -> Result<Self, Self::Error> {
//         let currency = item.router_data.request.currency.to_string();
//         let payment_id = &item.router_data.request.connector_transaction_id;
//         if payment_id.is_empty() {
//             return Err(errors::ConnectorError::MissingRequiredField { field_name: "connector_transaction_id" }.into());
//         }
//         Ok(Self {
//             amount: item.amount.to_owned(),
//             currency,
//             payment_id: payment_id.to_string(),
//         })
//     }
// }

// // Type definition for Refund Response

// #[allow(dead_code)]
// #[derive(Debug, Copy, Serialize, Default, Deserialize, Clone)]
// pub enum RefundStatus {
//     Succeeded,
//     Failed,
//     #[default]
//     Processing,
// }

// impl RefundStatus {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             RefundStatus::Succeeded => "succeeded",
//             RefundStatus::Failed => "failed",
//             RefundStatus::Processing => "processing",
//         }
//     }
// }

// impl From<RefundStatus> for enums::RefundStatus {
//     fn from(item: RefundStatus) -> Self {
//         match item {
//             RefundStatus::Succeeded => Self::Success,
//             RefundStatus::Failed => Self::Failure,
//             RefundStatus::Processing => Self::Pending,
//             //TODO: Review mapping
//         }
//     }
// }

// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
// pub struct RefundResponse {
//     pub id: String,
//     pub status: RefundStatus,
//     pub amount: Option<StringMinorUnit>,
//     pub currency: Option<String>,
//     pub payment_id: Option<String>,
// }

// impl TryFrom<RefundsResponseRouterData<Execute, RefundResponse>> for RefundsRouterData<Execute> {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(
//         item: RefundsResponseRouterData<Execute, RefundResponse>,
//     ) -> Result<Self, Self::Error> {
//         let status = match item.response.status.as_str() {
//             "succeeded" => enums::RefundStatus::Success,
//             "failed" => enums::RefundStatus::Failure,
//             _ => enums::RefundStatus::Pending,
//         };
//         Ok(Self {
//             response: Ok(RefundsResponseData {
//                 connector_refund_id: item.response.id.clone(),
//                 refund_status: status,
//             }),
//             ..item.data
//         })
//     }
// }

// impl TryFrom<RefundsResponseRouterData<RSync, RefundResponse>> for RefundsRouterData<RSync> {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(
//         item: RefundsResponseRouterData<RSync, RefundResponse>,
//     ) -> Result<Self, Self::Error> {
//         let status = match item.response.status.as_str() {
//             "succeeded" => enums::RefundStatus::Success,
//             "failed" => enums::RefundStatus::Failure,
//             _ => enums::RefundStatus::Pending,
//         };
//         Ok(Self {
//             response: Ok(RefundsResponseData {
//                 connector_refund_id: item.response.id.clone(),
//                 refund_status: status,
//             }),
//             ..item.data
//         })
//     }
// }

    


// //TODO: Fill the struct with respective fields

    



use common_enums::enums;
use common_utils::{
    id_type,
    pii::{Email, SecretSerdeValue},
    types::MinorUnit,
};
use hyperswitch_domain_models::{
    payment_method_data::PaymentMethodData,
    router_data::{ConnectorAuthType, ErrorResponse, PaymentMethodToken, RouterData},
    router_flow_types::{
        payments,
        refunds::{Execute, RSync},
    },
    router_request_types::ResponseId,
    router_response_types::{PaymentsResponseData, RefundsResponseData},
    types,
};
use hyperswitch_interfaces::{
    consts::{NO_ERROR_CODE, NO_ERROR_MESSAGE},
    errors,
};
use masking::{ExposeInterface, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    types::{RefundsResponseRouterData, ResponseRouterData},
    utils::{self, CardData as _, PaymentsAuthorizeRequestData, RouterData as _},
};

pub struct DemopayRouterData<T> {
    pub amount: MinorUnit,
    pub router_data: T,
}

impl<T> TryFrom<(MinorUnit, T)> for DemopayRouterData<T> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from((amount, item): (MinorUnit, T)) -> Result<Self, Self::Error> {
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

pub struct DemopayAuthType {
    pub(super) api_key: Secret<String>,
    pub(super) public_api_key: Secret<String>,
}

impl TryFrom<&ConnectorAuthType> for DemopayAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::BodyKey { api_key, key1 } => Ok(Self {
                api_key: api_key.to_owned(),
                public_api_key: key1.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemopayTokenRequestIntent {
    ChargeAndStore,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemopayStrongAuthRule {
    UseScaIfAvailableAuth,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemopayTokenRequest {
    number: cards::CardNumber,
    month: Secret<String>,
    year: Secret<String>,
    cvv: Secret<String>,
    pkey: Secret<String>,
    recurring: Option<bool>,
    intent: Option<DemopayTokenRequestIntent>,
    strong_authentication_rule: Option<DemopayStrongAuthRule>,
}

impl TryFrom<&types::TokenizationRouterData> for DemopayTokenRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::TokenizationRouterData) -> Result<Self, Self::Error> {
        match item.request.payment_method_data.clone() {
            PaymentMethodData::Card(ccard) => {
                let connector_auth = &item.connector_auth_type;
                let auth_type = DemopayAuthType::try_from(connector_auth)?;
                Ok(Self {
                    number: ccard.card_number.clone(),
                    month: ccard.card_exp_month.clone(),
                    year: ccard.get_card_expiry_year_2_digit()?,
                    cvv: ccard.card_cvc,
                    pkey: auth_type.public_api_key,
                    recurring: None,
                    intent: None,
                    strong_authentication_rule: None,
                })
            }
            PaymentMethodData::Wallet(_)
            | PaymentMethodData::CardRedirect(_)
            | PaymentMethodData::PayLater(_)
            | PaymentMethodData::BankRedirect(_)
            | PaymentMethodData::BankDebit(_)
            | PaymentMethodData::BankTransfer(_)
            | PaymentMethodData::Crypto(_)
            | PaymentMethodData::MandatePayment
            | PaymentMethodData::Reward
            | PaymentMethodData::RealTimePayment(_)
            | PaymentMethodData::MobilePayment(_)
            | PaymentMethodData::Upi(_)
            | PaymentMethodData::Voucher(_)
            | PaymentMethodData::GiftCard(_)
            | PaymentMethodData::OpenBanking(_)
            | PaymentMethodData::CardToken(_)
            | PaymentMethodData::NetworkToken(_)
            | PaymentMethodData::CardDetailsForNetworkTransactionId(_) => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Demopay"),
                )
                .into())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DemopayTokenResponse {
    id: Secret<String>,
    recurring: Option<bool>,
}

impl<T>
    TryFrom<
        ResponseRouterData<
            payments::PaymentMethodToken,
            DemopayTokenResponse,
            T,
            PaymentsResponseData,
        >,
    > for RouterData<payments::PaymentMethodToken, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: ResponseRouterData<
            payments::PaymentMethodToken,
            DemopayTokenResponse,
            T,
            PaymentsResponseData,
        >,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(PaymentsResponseData::TokenizationResponse {
                token: item.response.id.expose(),
            }),
            ..item.data
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DemopayCustomerObject {
    handle: Option<id_type::CustomerId>,
    email: Option<Email>,
    address: Option<Secret<String>>,
    address2: Option<Secret<String>>,
    city: Option<String>,
    country: Option<common_enums::CountryAlpha2>,
    first_name: Option<Secret<String>>,
    last_name: Option<Secret<String>>,
}

#[derive(Debug, Serialize)]
pub struct DemopayPaymentsRequest {
    handle: String,
    amount: MinorUnit,
    source: Secret<String>,
    currency: common_enums::Currency,
    customer: DemopayCustomerObject,
    metadata: Option<SecretSerdeValue>,
    settle: bool,
}

impl TryFrom<&DemopayRouterData<&types::PaymentsAuthorizeRouterData>> for DemopayPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: &DemopayRouterData<&types::PaymentsAuthorizeRouterData>,
    ) -> Result<Self, Self::Error> {
        if item.router_data.is_three_ds() {
            return Err(errors::ConnectorError::NotImplemented(
                "Three_ds payments through Demopay".to_string(),
            )
            .into());
        };
        let source = match item.router_data.get_payment_method_token()? {
            PaymentMethodToken::Token(pm_token) => Ok(pm_token),
            _ => Err(errors::ConnectorError::MissingRequiredField {
                field_name: "payment_method_token",
            }),
        }?;
        Ok(Self {
            handle: item.router_data.connector_request_reference_id.clone(),
            amount: item.amount,
            source,
            currency: item.router_data.request.currency,
            customer: DemopayCustomerObject {
                handle: item.router_data.customer_id.clone(),
                email: item.router_data.request.email.clone(),
                address: item.router_data.get_optional_billing_line1(),
                address2: item.router_data.get_optional_billing_line2(),
                city: item.router_data.get_optional_billing_city(),
                country: item.router_data.get_optional_billing_country(),
                first_name: item.router_data.get_optional_billing_first_name(),
                last_name: item.router_data.get_optional_billing_last_name(),
            },
            metadata: item.router_data.request.metadata.clone().map(Into::into),
            settle: item.router_data.request.is_auto_capture()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DemopayPaymentState {
    Created,
    Authorized,
    Pending,
    Settled,
    Failed,
    Cancelled,
}

impl From<DemopayPaymentState> for enums::AttemptStatus {
    fn from(item: DemopayPaymentState) -> Self {
        match item {
            DemopayPaymentState::Created | DemopayPaymentState::Pending => Self::Pending,
            DemopayPaymentState::Authorized => Self::Authorized,
            DemopayPaymentState::Settled => Self::Charged,
            DemopayPaymentState::Failed => Self::Failure,
            DemopayPaymentState::Cancelled => Self::Voided,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemopayPaymentsResponse {
    state: DemopayPaymentState,
    handle: String,
    error: Option<String>,
    error_state: Option<String>,
}

impl<F, T> TryFrom<ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        let error_response = if item.response.error.is_some() || item.response.error_state.is_some()
        {
            Some(ErrorResponse {
                code: item
                    .response
                    .error_state
                    .clone()
                    .unwrap_or(NO_ERROR_CODE.to_string()),
                message: item
                    .response
                    .error_state
                    .unwrap_or(NO_ERROR_MESSAGE.to_string()),
                reason: item.response.error,
                status_code: item.http_code,
                attempt_status: None,
                connector_transaction_id: Some(item.response.handle.clone()),
                network_advice_code: None,
                network_decline_code: None,
                network_error_message: None,
            })
        } else {
            None
        };
        let payments_response = PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(item.response.handle.clone()),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(item.response.handle),
            incremental_authorization_allowed: None,
            charges: None,
        };
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.state),
            response: error_response.map_or_else(|| Ok(payments_response), Err),
            ..item.data
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DemopayCaptureRequest {
    amount: MinorUnit,
}

impl TryFrom<&DemopayRouterData<&types::PaymentsCaptureRouterData>> for DemopayCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: &DemopayRouterData<&types::PaymentsCaptureRouterData>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: item.amount,
        })
    }
}

// Type definition for RefundRequest
#[derive(Debug, Serialize)]
pub struct DemopayRefundRequest {
    pub invoice: String,
    pub amount: MinorUnit,
    pub text: Option<String>,
}

impl<F> TryFrom<&DemopayRouterData<&types::RefundsRouterData<F>>> for DemopayRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: &DemopayRouterData<&types::RefundsRouterData<F>>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: item.amount,
            invoice: item.router_data.request.connector_transaction_id.clone(),
            text: item.router_data.request.reason.clone(),
        })
    }
}

// Type definition for Refund Response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefundState {
    Refunded,
    Failed,
    Processing,
}

impl From<RefundState> for enums::RefundStatus {
    fn from(item: RefundState) -> Self {
        match item {
            RefundState::Refunded => Self::Success,
            RefundState::Failed => Self::Failure,
            RefundState::Processing => Self::Pending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefundResponse {
    id: String,
    state: RefundState,
}

impl TryFrom<RefundsResponseRouterData<Execute, RefundResponse>>
    for types::RefundsRouterData<Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.state),
            }),
            ..item.data
        })
    }
}

impl TryFrom<RefundsResponseRouterData<RSync, RefundResponse>> for types::RefundsRouterData<RSync> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<RSync, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.state),
            }),
            ..item.data
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DemopayErrorResponse {
    pub code: Option<i32>,
    pub error: String,
    pub message: Option<String>,
}


