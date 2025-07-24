pub mod transformers;

use error_stack::{report, ResultExt};
use masking::{ExposeInterface, Mask};
use std::time::Duration;

use common_utils::{
    errors::CustomResult,
    ext_traits::BytesExt,
    types::{AmountConvertor, StringMinorUnit, StringMinorUnitForConnector},
    request::{Method, Request, RequestBuilder, RequestContent},
};

use hyperswitch_domain_models::{
    router_data::{AccessToken, ConnectorAuthType, ErrorResponse, RouterData},
    router_flow_types::{
        access_token_auth::AccessTokenAuth,
        payments::{
            Authorize, Capture, PSync, PaymentMethodToken, Session,
            SetupMandate, Void,
        },
        refunds::{Execute, RSync},
    },
    router_request_types::{
        AccessTokenRequestData, PaymentMethodTokenizationData,
        PaymentsAuthorizeData, PaymentsCancelData, PaymentsCaptureData, PaymentsSessionData,
        PaymentsSyncData, RefundsData, SetupMandateRequestData,
    },
    router_response_types::{PaymentsResponseData, RefundsResponseData},
    types::{
        PaymentsAuthorizeRouterData,
        PaymentsCaptureRouterData, PaymentsSyncRouterData, RefundSyncRouterData, RefundsRouterData,
    },
};
use hyperswitch_interfaces::{
    api::{self, ConnectorCommon, ConnectorCommonExt, ConnectorIntegration, ConnectorValidation, ConnectorSpecifications},
    configs::Connectors,
    errors,
    events::connector_api_logs::ConnectorEvent,
    types::{self, Response},
    webhooks,
};
use std::sync::LazyLock;

use common_enums::enums;
use hyperswitch_interfaces::api::ConnectorSpecifications;
use hyperswitch_domain_models::router_response_types::{ConnectorInfo, SupportedPaymentMethods};
use crate::{
    constants::headers,
    types::ResponseRouterData,
    utils,
};
use hyperswitch_domain_models::payment_method_data::PaymentMethodData;
use router_env::logger;

use transformers as demopay;

#[derive(Clone)]
pub struct Demopay {
    amount_converter: &'static (dyn AmountConvertor<Output = StringMinorUnit> + Sync)
}

impl Demopay {
    pub fn new() -> &'static Self {
        static INSTANCE: Demopay = Demopay {
            amount_converter: &StringMinorUnitForConnector
        };
        &INSTANCE
    }
}

impl api::Payment for Demopay {}
impl api::PaymentSession for Demopay {}
impl api::ConnectorAccessToken for Demopay {}
impl api::MandateSetup for Demopay {}
impl api::PaymentAuthorize for Demopay {}
impl api::PaymentSync for Demopay {}
impl api::PaymentCapture for Demopay {}
impl api::PaymentVoid for Demopay {}
impl api::Refund for Demopay {}
impl api::RefundExecute for Demopay {}
impl api::RefundSync for Demopay {}
impl api::PaymentToken for Demopay {}

impl<Flow, Request, Response> ConnectorCommonExt<Flow, Request, Response> for Demopay
where
    Self: ConnectorIntegration<Flow, Request, Response>,{
    fn build_headers(
        &self,
        req: &RouterData<Flow, Request, Response>,
        _connectors: &Connectors,
    ) -> CustomResult<Vec<(String, masking::Maskable<String>)>, errors::ConnectorError> {
        let mut header = vec![(
            headers::CONTENT_TYPE.to_string(),
            self.get_content_type().to_string().into(),
        )];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        header.append(&mut api_key);
        Ok(header)
    }
}

impl ConnectorCommon for Demopay {
    fn id(&self) -> &'static str {
        "demopay"
    }

    fn get_currency_unit(&self) -> api::CurrencyUnit {
        api::CurrencyUnit::Minor
    }

    fn common_get_content_type(&self) -> &'static str {
        "application/json"
    }

    fn base_url<'a>(&self, connectors: &'a Connectors) -> &'a str {
        connectors.demopay.base_url.as_ref()
    }

    fn get_auth_header(&self, auth_type:&ConnectorAuthType)-> CustomResult<Vec<(String,masking::Maskable<String>)>,errors::ConnectorError> {
        let auth = demopay::DemoPayAuthType::try_from(auth_type)
            .change_context(errors::ConnectorError::FailedToObtainAuthType)?;
        Ok(vec![(headers::AUTHORIZATION.to_string(), format!("Bearer {}", auth.api_key.expose()).into_masked())])
    }

    fn build_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        let response: demopay::DemoPayErrorResponse = res
            .response
            .parse_struct("DemoPayErrorResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;

        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);

        Ok(ErrorResponse {
            status_code: res.status_code,
            code: response.code,
            message: response.message,
            reason: response.reason,
            attempt_status: None,
            connector_transaction_id: None,
            network_advice_code: response.network_advice_code,
            network_decline_code: response.network_decline_code,
            network_error_message: response.network_error_message,
        })
    }
}

impl ConnectorValidation for Demopay {
    fn validate_mandate_payment(
        &self,
        _pm_type: Option<enums::PaymentMethodType>,
        _pm_data: PaymentMethodData,
    ) -> CustomResult<(), errors::ConnectorError> {
        Ok(())
    }

    fn validate_psync_reference_id(
        &self,
        _data: &PaymentsSyncData,
        _is_three_ds: bool,
        _status: enums::AttemptStatus,
        _connector_meta_data: Option<common_utils::pii::SecretSerdeValue>,
    ) -> CustomResult<(), errors::ConnectorError> {
        Ok(())
    }
}

impl
    ConnectorIntegration<
        Authorize,
        PaymentsAuthorizeData,
        PaymentsResponseData,
    > for Demopay {
    fn get_headers(&self, req: &PaymentsAuthorizeRouterData, connectors: &Connectors,) -> CustomResult<Vec<(String, masking::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &PaymentsAuthorizeRouterData,
        connectors: &Connectors,) -> common_utils::errors::CustomResult<String,errors::ConnectorError> {
        Ok(format!("{}/pay", self.base_url(connectors)))
    }

    fn get_request_body(&self, req: &PaymentsAuthorizeRouterData, _connectors: &Connectors,) -> common_utils::errors::CustomResult<RequestContent, errors::ConnectorError> {
        let amount = utils::convert_amount(
            self.amount_converter,
            req.request.minor_amount,
            req.request.currency,
        )?;

        let connector_router_data = demopay::DemoPayRouterData::from((amount, req));
        let connector_req = demopay::DemoPayPaymentsRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(
        &self,
        req: &PaymentsAuthorizeRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        let request = Some(
            RequestBuilder::new()
                .method(Method::Post)
                .url(&types::PaymentsAuthorizeType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsAuthorizeType::get_headers(self, req, connectors)?)
                .set_body(types::PaymentsAuthorizeType::get_request_body(self, req, connectors)?)
                .build(),
        );
        
        // Log the request for debugging
        if let Some(req) = &request {
            logger::debug!(?req);
        }
        
        Ok(request)
    }

    fn handle_response(
        &self,
        data: &PaymentsAuthorizeRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<PaymentsAuthorizeRouterData, errors::ConnectorError> {
        let response: demopay::DemoPayPaymentsResponse = res.response
            .parse_struct("DemoPayPaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        
        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);
        
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl
    ConnectorIntegration<PSync, PaymentsSyncData, PaymentsResponseData>
    for Demopay
{
    fn get_headers(
        &self,
        req: &PaymentsSyncRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Vec<(String, masking::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &PaymentsSyncRouterData,
        connectors: &Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let connector_payment_id = req.request.connector_transaction_id.clone();
        Ok(format!(
            "{}/status/{}",
            self.base_url(connectors),
            connector_payment_id
        ))
    }

    fn build_request(
        &self,
        req: &PaymentsSyncRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        let request = Some(
            RequestBuilder::new()
                .method(Method::Get)
                .url(&types::PaymentsSyncType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsSyncType::get_headers(self, req, connectors)?)
                .build(),
        );
        
        // Log the request for debugging
        if let Some(req) = &request {
            logger::debug!(?req);
        }
        
        Ok(request)
    }

    fn handle_response(
        &self,
        data: &PaymentsSyncRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<PaymentsSyncRouterData, errors::ConnectorError> {
        let response: demopay::DemoPayPaymentsResponse = res
            .response
            .parse_struct("DemoPayPaymentsSyncResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        
        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);
        
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl
    ConnectorIntegration<
        Capture,
        PaymentsCaptureData,
        PaymentsResponseData,
    > for Demopay
{
    fn get_headers(
        &self,
        req: &PaymentsCaptureRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Vec<(String, masking::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &PaymentsCaptureRouterData,
        connectors: &Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!("{}/capture", self.base_url(connectors)))
    }

    fn get_request_body(
        &self,
        req: &PaymentsCaptureRouterData,
        _connectors: &Connectors,
    ) -> CustomResult<RequestContent, errors::ConnectorError> {
        let amount = utils::convert_amount(
            self.amount_converter,
            req.request.minor_amount,
            req.request.currency,
        )?;
        
        let connector_router_data = demopay::DemoPayRouterData::from((amount, req));
        let connector_req = demopay::DemoPayCaptureRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(
        &self,
        req: &PaymentsCaptureRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        let request = Some(
            RequestBuilder::new()
                .method(Method::Post)
                .url(&types::PaymentsCaptureType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsCaptureType::get_headers(self, req, connectors)?)
                .set_body(types::PaymentsCaptureType::get_request_body(
                    self, req, connectors,
                )?)
                .build(),
        );
        
        // Log the request for debugging
        if let Some(req) = &request {
            logger::debug!(?req);
        }
        
        Ok(request)
    }

    fn handle_response(
        &self,
        data: &PaymentsCaptureRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<PaymentsCaptureRouterData, errors::ConnectorError> {
        let response: demopay::DemoPayPaymentsResponse = res
            .response
            .parse_struct("DemoPayPaymentsCaptureResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        
        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);
        
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl
    ConnectorIntegration<
        Void,
        PaymentsCancelData,
        PaymentsResponseData,
    > for Demopay
{
}

impl ConnectorIntegration<Execute, RefundsData, RefundsResponseData> for Demopay {
    fn get_headers(&self, req: &RefundsRouterData<Execute>, connectors: &Connectors,) -> CustomResult<Vec<(String,masking::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &RefundsRouterData<Execute>,
        connectors: &Connectors,) -> CustomResult<String,errors::ConnectorError> {
        Ok(format!("{}/refund", self.base_url(connectors)))
    }

    fn get_request_body(&self, req: &RefundsRouterData<Execute>, _connectors: &Connectors,) -> CustomResult<RequestContent, errors::ConnectorError> {
        let refund_amount = utils::convert_amount(
            self.amount_converter,
            req.request.refund_amount,
            req.request.currency,
        )?;
        
        let connector_router_data = demopay::DemoPayRouterData::from((
            refund_amount,
            req,
        ));
        let connector_req = demopay::DemoPayRefundRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(
        &self,
        req: &RefundsRouterData<Execute>,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>,errors::ConnectorError> {
        let request = Some(
            RequestBuilder::new()
                .method(Method::Post)
                .url(&types::RefundExecuteType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::RefundExecuteType::get_headers(
                    self, req, connectors,
                )?)
                .set_body(types::RefundExecuteType::get_request_body(
                    self, req, connectors,
                )?)
                .build(),
        );
        
        // Log the request for debugging
        if let Some(req) = &request {
            logger::debug!(?req);
        }
        
        Ok(request)
    }

    fn handle_response(
        &self,
        data: &RefundsRouterData<Execute>,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<RefundsRouterData<Execute>,errors::ConnectorError> {
        let response: demopay::RefundResponse = res
            .response
            .parse_struct("RefundResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        
        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);
        
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl
    ConnectorIntegration<RSync, RefundsData, RefundsResponseData> for Demopay {
    fn get_headers(&self, req: &RefundSyncRouterData,connectors: &Connectors,) -> CustomResult<Vec<(String, masking::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &RefundSyncRouterData,
        connectors: &Connectors,) -> CustomResult<String,errors::ConnectorError> {
        let refund_id = req.request.connector_refund_id.clone();
        Ok(format!("{}/status/{}", self.base_url(connectors), refund_id))
    }

    fn build_request(
        &self,
        req: &RefundSyncRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        let request = Some(
            RequestBuilder::new()
                .method(Method::Get)
                .url(&types::RefundSyncType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::RefundSyncType::get_headers(self, req, connectors)?)
                .build(),
        );
        
        // Log the request for debugging
        if let Some(req) = &request {
            logger::debug!(?req);
        }
        
        Ok(request)
    }

    fn handle_response(
        &self,
        data: &RefundSyncRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<RefundSyncRouterData,errors::ConnectorError,> {
        let response: demopay::RefundResponse = res
            .response
            .parse_struct("RefundSyncResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        
        event_builder.map(|i| i.set_response_body(&response));
        logger::info!(connector_response=?response);
        
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

#[async_trait::async_trait]
impl webhooks::IncomingWebhook for Demopay {
    fn get_webhook_object_reference_id(
        &self,
        _request: &webhooks::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api_models::webhooks::ObjectReferenceId, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_event_type(
        &self,
        _request: &webhooks::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api_models::webhooks::IncomingWebhookEvent, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_resource_object(
        &self,
        _request: &webhooks::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<Box<dyn masking::ErasedMaskSerialize>, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }
}

static DEMOPAY_SUPPORTED_PAYMENT_METHODS: LazyLock<SupportedPaymentMethods> =
    LazyLock::new(|| {
        let mut payment_methods = SupportedPaymentMethods::new();
        payment_methods.add_payment_method(enums::PaymentMethod::Wallet);
        payment_methods
    });

impl ConnectorSpecifications for Demopay {
    fn get_connector_about(&self) -> Option<&'static ConnectorInfo> {
        Some(&ConnectorInfo {
            name: "DemoPay",
            description: "DemoPay is a sample payment connector for demonstration purposes.",
            connector_type: enums::ConnectorType::PaymentProcessor,
            connector_id: "demopay",
        })
    }

    fn get_supported_payment_methods(&self) -> Option<&'static SupportedPaymentMethods> {
        Some(&DEMOPAY_SUPPORTED_PAYMENT_METHODS)
    }

    fn get_supported_webhook_flows(&self) -> Option<&'static [enums::EventClass]> {
        None
    }
}
