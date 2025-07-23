use common_enums::enums;
use common_utils::types::StringMinorUnit;
use hyperswitch_domain_models::{
    payment_method_data::PaymentMethodData,
    router_data::{ConnectorAuthType, RouterData},
    router_flow_types::refunds::{Execute, RSync},
    router_request_types::ResponseId,
    router_response_types::{PaymentsResponseData, RefundsResponseData},
    types::{PaymentsAuthorizeRouterData, RefundsRouterData},
};
use hyperswitch_interfaces::errors;
use masking::Secret;
use serde::{Deserialize, Serialize};

use crate::types::{RefundsResponseRouterData, ResponseRouterData};


#[derive(Debug, Clone)]
pub struct DemopayRouterData<T> {
    pub amount: StringMinorUnit,
    pub router_data: T,
}

impl<T> From<(StringMinorUnit, T)> for DemopayRouterData<T> {
    fn from((amount, item): (StringMinorUnit, T)) -> Self {
        Self {
            amount,
            router_data: item,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DemopayPaymentsRequest {
    pub amount: StringMinorUnit,
    pub currency: String,
    pub wallet_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DemopayPaymentsResponse {
    pub txn_id: String,
    pub status: String, // e.g., "authorized", "captured", "failed"
    pub message: Option<String>,
    pub amount: Option<StringMinorUnit>,
    pub currency: Option<String>,
}

impl TryFrom<&DemopayRouterData<&PaymentsAuthorizeRouterData>> for DemopayPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: &DemopayRouterData<&PaymentsAuthorizeRouterData>,
    ) -> Result<Self, Self::Error> {
        let (wallet_id, currency) = match &item.router_data.request.payment_method_data {
            PaymentMethodData::Wallet(wallet_data) => {
                // Example: extract wallet_id and currency from wallet_data
                // (Replace with actual extraction logic as per your WalletData structure)
                ("demo_wallet_id".to_string(), item.router_data.request.currency.to_string())
            },
            _ => return Err(errors::ConnectorError::NotImplemented(
                "Only wallet payments supported for DemoPay".to_string(),
            )
            .into()),
        };
        Ok(Self {
            amount: item.amount.clone(),
            currency,
            wallet_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemopayErrorResponse {
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
}

// Auth Struct
pub struct DemopayAuthType {
    pub(super) api_key: Secret<String>,
}

impl TryFrom<&ConnectorAuthType> for DemopayAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::HeaderKey { api_key } => Ok(Self {
                api_key: api_key.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}
// PaymentsResponse
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DemopayPaymentStatus {
    Authorized,
    Captured,
    Failed,
    #[default]
    Processing,
}

// Always-successful capture response for Demopay
use hyperswitch_domain_models::types::PaymentsCaptureRouterData;

impl TryFrom<&DemopayRouterData<&PaymentsCaptureRouterData>> for DemopayPaymentsResponse {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemopayRouterData<&PaymentsCaptureRouterData>) -> Result<Self, Self::Error> {
        Ok(Self {
            txn_id: item.router_data.request.connector_transaction_id.clone(),
            status: "captured".to_string(),
            message: Some("Payment captured successfully (stub)".to_string()),
            amount: Some(item.amount.clone()),
            currency: Some(item.router_data.request.currency.to_string()),
        })
    }
}

// Always-successful refund response for Demopay

impl TryFrom<&DemopayRouterData<&RefundsRouterData<Execute>>> for RefundResponse {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemopayRouterData<&RefundsRouterData<Execute>>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: format!("refund-{}", item.router_data.request.connector_transaction_id),
            status: RefundStatus::Succeeded,
            amount: Some(item.amount.clone()),
            currency: Some(item.router_data.request.currency.to_string()),
            payment_id: Some(item.router_data.request.connector_transaction_id.clone()),
        })
    }
}

impl TryFrom<&DemopayRouterData<&RefundsRouterData<RSync>>> for RefundResponse {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemopayRouterData<&RefundsRouterData<RSync>>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: format!("refund-{}", item.router_data.request.connector_transaction_id),
            status: RefundStatus::Succeeded,
            amount: Some(item.amount.clone()),
            currency: Some(item.router_data.request.currency.to_string()),
            payment_id: Some(item.router_data.request.connector_transaction_id.clone()),
        })
    }
}


impl From<DemopayPaymentStatus> for common_enums::AttemptStatus {
    fn from(item: DemopayPaymentStatus) -> Self {
        match item {
            DemopayPaymentStatus::Authorized => Self::Authorized,
            DemopayPaymentStatus::Captured => Self::Charged,
            DemopayPaymentStatus::Failed => Self::Failure,
            DemopayPaymentStatus::Processing => Self::Pending,
        }
    }
}

impl<F, T> TryFrom<ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: ResponseRouterData<F, DemopayPaymentsResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        // Map DemoPay status string to AttemptStatus
        let status = match item.response.status.as_str() {
            "authorized" => common_enums::AttemptStatus::Authorized,
            "captured" => common_enums::AttemptStatus::Charged,
            "failed" => common_enums::AttemptStatus::Failure,
            _ => common_enums::AttemptStatus::Pending,
        };
        Ok(Self {
            status,
            response: Ok(PaymentsResponseData::TransactionResponse {
                resource_id: ResponseId::ConnectorTransactionId(item.response.txn_id.clone()),
                redirection_data: Box::new(None),
                mandate_reference: Box::new(None),
                connector_metadata: None,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
                charges: None,
            }),
            ..item.data
        })
    }
}

#[derive(Default, Debug, Serialize)]
pub struct DemopayRefundRequest {
    pub amount: StringMinorUnit,
    pub currency: String,
    pub payment_id: String,
}

impl<F> TryFrom<&DemopayRouterData<&RefundsRouterData<F>>> for DemopayRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemopayRouterData<&RefundsRouterData<F>>) -> Result<Self, Self::Error> {
        let currency = item.router_data.request.currency.to_string();
        let payment_id = &item.router_data.request.connector_transaction_id;
        if payment_id.is_empty() {
            return Err(errors::ConnectorError::MissingRequiredField { field_name: "connector_transaction_id" }.into());
        }
        Ok(Self {
            amount: item.amount.to_owned(),
            currency,
            payment_id: payment_id.to_string(),
        })
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Copy, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl RefundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefundStatus::Succeeded => "succeeded",
            RefundStatus::Failed => "failed",
            RefundStatus::Processing => "processing",
        }
    }
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub id: String,
    pub status: RefundStatus,
    pub amount: Option<StringMinorUnit>,
    pub currency: Option<String>,
    pub payment_id: Option<String>,
}

impl TryFrom<RefundsResponseRouterData<Execute, RefundResponse>> for RefundsRouterData<Execute> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        let status = match item.response.status.as_str() {
            "succeeded" => enums::RefundStatus::Success,
            "failed" => enums::RefundStatus::Failure,
            _ => enums::RefundStatus::Pending,
        };
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.clone(),
                refund_status: status,
            }),
            ..item.data
        })
    }
}

impl TryFrom<RefundsResponseRouterData<RSync, RefundResponse>> for RefundsRouterData<RSync> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<RSync, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        let status = match item.response.status.as_str() {
            "succeeded" => enums::RefundStatus::Success,
            "failed" => enums::RefundStatus::Failure,
            _ => enums::RefundStatus::Pending,
        };
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.clone(),
                refund_status: status,
            }),
            ..item.data
        })
    }
}

    


//TODO: Fill the struct with respective fields






    


