use common_enums::enums;
use serde::{Deserialize, Serialize};
use masking::Secret;
use common_utils::types::{StringMinorUnit};
use hyperswitch_domain_models::{
    payment_method_data::PaymentMethodData,
    router_data::{ConnectorAuthType, RouterData},
    router_flow_types::refunds::{Execute, RSync},
    router_request_types::ResponseId,
    router_response_types::{PaymentsResponseData, RefundsResponseData},
    types::{PaymentsAuthorizeRouterData, PaymentsCaptureRouterData, PaymentsSyncRouterData, RefundSyncRouterData, RefundsRouterData},
};
use hyperswitch_interfaces::errors;
use crate::types::{RefundsResponseRouterData, ResponseRouterData};
use router_env::logger;

// DemoPay Router Data
pub struct DemoPayRouterData<T> {
    pub amount: StringMinorUnit,
    pub router_data: T,
}

impl<T>
    From<(
        StringMinorUnit,
        T,
    )> for DemoPayRouterData<T>
{
    fn from(
        (amount, item): (
            StringMinorUnit,
            T,
        ),
    ) -> Self {
        Self {
            amount,
            router_data: item,
        }
    }
}

// DemoPay Payment Request
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct DemoPayPaymentsRequest {
    pub amount: StringMinorUnit,
    pub currency: String,
    pub wallet_id: String,
    pub transaction_id: String,
}

// DemoPay Capture Request
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct DemoPayCaptureRequest {
    pub amount: StringMinorUnit,
    pub transaction_id: String,
}

// DemoPay Refund Request
#[derive(Default, Debug, Serialize)]
pub struct DemoPayRefundRequest {
    pub amount: StringMinorUnit,
    pub transaction_id: String,
    pub refund_id: String,
}

// DemoPay Auth Type
pub struct DemoPayAuthType {
    pub(super) api_key: Secret<String>
}

impl TryFrom<&ConnectorAuthType> for DemoPayAuthType {
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

// DemoPay Payment Status
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DemoPayPaymentStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
    Authorized,
    Captured,
    Voided,
}

impl From<DemoPayPaymentStatus> for enums::AttemptStatus {
    fn from(item: DemoPayPaymentStatus) -> Self {
        match item {
            DemoPayPaymentStatus::Succeeded => Self::Charged,
            DemoPayPaymentStatus::Failed => Self::Failure,
            DemoPayPaymentStatus::Processing => Self::Pending,
            DemoPayPaymentStatus::Authorized => Self::Authorized,
            DemoPayPaymentStatus::Captured => Self::Charged,
            DemoPayPaymentStatus::Voided => Self::Voided,
        }
    }
}

// DemoPay Payments Response
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemoPayPaymentsResponse {
    pub status: DemoPayPaymentStatus,
    pub id: String,
    pub amount: Option<StringMinorUnit>,
    pub currency: Option<String>,
}

impl<F, T> TryFrom<ResponseRouterData<F, DemoPayPaymentsResponse, T, PaymentsResponseData>> for RouterData<F, T, PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: ResponseRouterData<F, DemoPayPaymentsResponse, T, PaymentsResponseData>) -> Result<Self, Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status),
            response: Ok(PaymentsResponseData::TransactionResponse {
                resource_id: ResponseId::ConnectorTransactionId(item.response.id),
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

// DemoPay Refund Status
#[derive(Debug, Copy, Serialize, Default, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
        }
    }
}

// DemoPay Refund Response
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub id: String,
    pub status: RefundStatus,
    pub amount: Option<StringMinorUnit>,
    pub transaction_id: String,
}

impl TryFrom<RefundsResponseRouterData<Execute, RefundResponse>>
    for RefundsRouterData<Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<RefundsResponseRouterData<RSync, RefundResponse>> for RefundsRouterData<RSync>
{
     type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: RefundsResponseRouterData<RSync, RefundResponse>) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
     }
}

// DemoPay Error Response
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct DemoPayErrorResponse {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
    pub network_advice_code: Option<String>,
    pub network_decline_code: Option<String>,
    pub network_error_message: Option<String>,
}

// Implementation for Payment Request
impl TryFrom<&DemoPayRouterData<&PaymentsAuthorizeRouterData>> for DemoPayPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemoPayRouterData<&PaymentsAuthorizeRouterData>) -> Result<Self, Self::Error> {
        match &item.router_data.request.payment_method_data {
            PaymentMethodData::Wallet(wallet_data) => {
                let wallet_id = wallet_data.token.clone().ok_or(
                    errors::ConnectorError::MissingRequiredField {
                        field_name: "wallet_id",
                    }
                )?;
                
                Ok(Self {
                    amount: item.amount.to_owned(),
                    currency: item.router_data.request.currency.to_string(),
                    wallet_id,
                    transaction_id: item.router_data.connector_request_reference_id.clone(),
                })
            },
            _ => Err(errors::ConnectorError::NotImplemented("Payment method not supported".to_string()).into()),
        }
    }
}

// Implementation for Capture Request
impl TryFrom<&DemoPayRouterData<&PaymentsCaptureRouterData>> for DemoPayCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemoPayRouterData<&PaymentsCaptureRouterData>) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: item.amount.to_owned(),
            transaction_id: item.router_data.connector_request_reference_id.clone(),
        })
    }
}

// Implementation for Refund Request
impl<F> TryFrom<&DemoPayRouterData<&RefundsRouterData<F>>> for DemoPayRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &DemoPayRouterData<&RefundsRouterData<F>>) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: item.amount.to_owned(),
            transaction_id: item.router_data.request.connector_transaction_id.clone(),
            refund_id: item.router_data.connector_request_reference_id.clone(),
        })
    }
}
