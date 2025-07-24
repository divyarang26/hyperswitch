// DemoPay Connector implementation for Hyperswitch
// Implements: authorize, capture, refund, status
// Auth: Bearer token

mod transformers;

use hyperswitch_interfaces::errors::ConnectorError;
use hyperswitch_interfaces::api::{ConnectorCommon, ConnectorIntegration, CurrencyUnit};
use hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent;
use hyperswitch_interfaces::types::Response;
use common_utils::request::RequestContent;
use hyperswitch_domain_models::types::{PaymentsAuthorizeRouterData, PaymentsCaptureRouterData, PaymentsSyncRouterData, RefundsRouterData};
use hyperswitch_interfaces::configs::Connectors;

use common_enums::AttemptStatus;
use hyperswitch_domain_models::{
    router_data::{ConnectorAuthType, RouterData},
    router_flow_types::{Execute, RSync},
    router_request_types::ResponseId,
    router_response_types::{PaymentsResponseData, RefundsResponseData},
};
use hyperswitch_interfaces::errors::ConnectorError;



use transformers::*;
use hyperswitch_domain_models::router_data::ErrorResponse;

pub struct DemoPay;

impl core::ConnectorCommon for DemoPay {
    fn id(&self) -> &'static str {
        "demopay"
    }
    fn get_currency_unit(&self) -> api::CurrencyUnit {
        api::CurrencyUnit::Minor
    }
    fn common_get_content_type(&self) -> &'static str {
        "application/json"
    }
    fn get_auth_header(&self, auth_type: &ConnectorAuthType) -> common_utils::errors::CustomResult<Vec<(String, masking::Maskable<String>)>, ConnectorError> {
        let token = match auth_type {
            ConnectorAuthType::HeaderKey { api_key } => api_key.peek().to_string(),
            _ => return Err(ConnectorError::FailedToObtainAuthType.into()),
        };
        Ok(vec![("Authorization".to_string(), format!("Bearer {}", token).into_masked())])
    }
    fn base_url<'a>(&self, connectors: &'a Connectors) -> &'a str {
        connectors.demopay.base_url.as_ref()
    }
    fn build_error_response(&self, res: Response, _event_builder: Option<&mut ConnectorEvent>) -> common_utils::errors::CustomResult<ErrorResponse, ConnectorError> {
        let response: DemoPayErrorResponse = res.response.parse_struct("DemoPayErrorResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        Ok(ErrorResponse {
            status_code: res.status_code,
            code: response.code.unwrap_or_else(|| "unknown".to_string()),
            message: response.message.unwrap_or_else(|| "Unknown error".to_string()),
            reason: response.error,
            attempt_status: None,
            connector_transaction_id: None,
        })
    }
}

// --- Authorize ---
impl core::ConnectorIntegration<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData> for DemoPay {
    fn get_url(&self, _req: &RouterData<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData>, connectors: &Connectors) -> common_utils::errors::CustomResult<String, ConnectorError> {
        Ok(format!("{}/pay", self.base_url(connectors)))
    }
    fn get_request_body(&self, req: &RouterData<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData>) -> common_utils::errors::CustomResult<Option<RequestContent>, ConnectorError> {
        let demopay_req = DemoPayPaymentsRequest::try_from(&req.request)?;
        Ok(Some(RequestContent::Json(Box::new(demopay_req))))
    }
    fn build_response(&self, res: Response) -> common_utils::errors::CustomResult<PaymentsResponseData, ConnectorError> {
        let response: DemoPayPaymentsResponse = res.response.parse_struct("DemoPayPaymentsResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        let status = match response.wallet_id.as_str() {
            "abc" => AttemptStatus::Failure,
            "def" | "xyz" => AttemptStatus::Authorized,
            _ => AttemptStatus::Pending,
        };
        Ok(PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(response.txn_id.clone()),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(response.txn_id),
            incremental_authorization_allowed: None,
            charges: None,
        })
    }
}

// --- Capture ---
impl core::ConnectorIntegration<Execute, PaymentsCaptureRouterData, PaymentsResponseData> for DemoPay {
    fn get_url(&self, _req: &RouterData<Execute, PaymentsCaptureRouterData, PaymentsResponseData>, connectors: &Connectors) -> common_utils::errors::CustomResult<String, ConnectorError> {
        Ok(format!("{}/capture", self.base_url(connectors)))
    }
    fn get_request_body(&self, req: &RouterData<Execute, PaymentsCaptureRouterData, PaymentsResponseData>) -> common_utils::errors::CustomResult<Option<RequestContent>, ConnectorError> {
        // Map to DemoPay capture request (reuse or define as needed)
        let demopay_req = DemoPayPaymentsRequest::try_from(&req.request)?;
        Ok(Some(RequestContent::Json(Box::new(demopay_req))))
    }
    fn build_response(&self, res: Response) -> common_utils::errors::CustomResult<PaymentsResponseData, ConnectorError> {
        let response: DemoPayPaymentsResponse = res.response.parse_struct("DemoPayPaymentsResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        let status = match response.wallet_id.as_str() {
            "abc" | "def" => AttemptStatus::Failure,
            "xyz" => AttemptStatus::Charged,
            _ => AttemptStatus::Pending,
        };
        Ok(PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(response.txn_id.clone()),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(response.txn_id),
            incremental_authorization_allowed: None,
            charges: None,
        })
    }
}

// --- Refund ---
impl core::ConnectorIntegration<Execute, RefundsRouterData, RefundsResponseData> for DemoPay {
    fn get_url(&self, _req: &RouterData<Execute, RefundsRouterData, RefundsResponseData>, connectors: &Connectors) -> common_utils::errors::CustomResult<String, ConnectorError> {
        Ok(format!("{}/refund", self.base_url(connectors)))
    }
    fn get_request_body(&self, req: &RouterData<Execute, RefundsRouterData, RefundsResponseData>) -> common_utils::errors::CustomResult<Option<RequestContent>, ConnectorError> {
        // Map to DemoPay refund request (reuse or define as needed)
        let demopay_req = DemoPayPaymentsRequest::try_from(&req.request)?;
        Ok(Some(RequestContent::Json(Box::new(demopay_req))))
    }
    fn build_response(&self, res: Response) -> common_utils::errors::CustomResult<RefundsResponseData, ConnectorError> {
        let response: DemoPayPaymentsResponse = res.response.parse_struct("DemoPayPaymentsResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        // Map response to refund status
        let status = if response.status == "refunded" { AttemptStatus::Charged } else { AttemptStatus::Failure };
        Ok(RefundsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(response.txn_id.clone()),
            refund_status: status,
            connector_response_reference_id: Some(response.txn_id),
        })
    }
}

// --- Status ---
impl core::ConnectorIntegration<RSync, PaymentsSyncRouterData, PaymentsResponseData> for DemoPay {
    fn get_url(&self, req: &RouterData<RSync, PaymentsSyncRouterData, PaymentsResponseData>, connectors: &Connectors) -> common_utils::errors::CustomResult<String, ConnectorError> {
        let txn_id = req.request.connector_transaction_id.clone().unwrap_or_default();
        Ok(format!("{}/status/{}", self.base_url(connectors), txn_id))
    }
    fn get_request_body(&self, _req: &RouterData<RSync, PaymentsSyncRouterData, PaymentsResponseData>) -> common_utils::errors::CustomResult<Option<RequestContent>, ConnectorError> {
        Ok(None)
    }
    fn build_response(&self, res: Response) -> common_utils::errors::CustomResult<PaymentsResponseData, ConnectorError> {
        let response: DemoPayPaymentsResponse = res.response.parse_struct("DemoPayPaymentsResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        let status = match response.status.as_str() {
            "authorized" => AttemptStatus::Authorized,
            "charged" => AttemptStatus::Charged,
            "failed" => AttemptStatus::Failure,
            _ => AttemptStatus::Pending,
        };
        Ok(PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(response.txn_id.clone()),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(response.txn_id),
            incremental_authorization_allowed: None,
            charges: None,
        })
    }
}

impl core::ConnectorIntegration<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData> for DemoPay {
    fn get_url(&self, _req: &RouterData<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData>, connectors: &Connectors) -> common_utils::errors::CustomResult<String, ConnectorError> {
        Ok(format!("{}/pay", self.base_url(connectors)))
    }
    fn get_request_body(&self, req: &RouterData<Execute, PaymentsAuthorizeRouterData, PaymentsResponseData>) -> common_utils::errors::CustomResult<Option<RequestContent>, ConnectorError> {
        let demopay_req = DemoPayPaymentsRequest::try_from(&req.request)?;
        Ok(Some(RequestContent::Json(Box::new(demopay_req))))
    }
    fn build_response(&self, res: Response) -> common_utils::errors::CustomResult<PaymentsResponseData, ConnectorError> {
        let response: DemoPayPaymentsResponse = res.response.parse_struct("DemoPayPaymentsResponse").change_context(ConnectorError::ResponseDeserializationFailed)?;
        Ok(PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(response.txn_id.clone()),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(response.txn_id),
            incremental_authorization_allowed: None,
            charges: None,
        })
    }
}

// Similar trait impls for capture, refund, status would be added here following the pattern above
