use async_trait::async_trait;
use common_utils::{ext_traits::ValueExt, pii::Email};
use error_stack::ResultExt;
use masking::ExposeInterface;

use crate::{
    core::{
        errors::{ConnectorErrorExt, RouterResult},
        fraud_check::{FeatureFrm, FraudCheckConnectorData, FrmData},
        payments::{self, flows::ConstructFlowSpecificData, helpers},
    },
    errors, services,
    types::{
        api::fraud_check as frm_api,
        domain,
        fraud_check::{FraudCheckResponseData, FraudCheckSaleData, FrmSaleRouterData},
        storage::enums as storage_enums,
        ConnectorAuthType, MerchantRecipientData, ResponseId, RouterData,
    },
    SessionState,
};

#[async_trait]
impl ConstructFlowSpecificData<frm_api::Sale, FraudCheckSaleData, FraudCheckResponseData>
    for FrmData
{
    #[cfg(feature = "v2")]
    async fn construct_router_data<'a>(
        &self,
        _state: &SessionState,
        _connector_id: &str,
        _merchant_context: &domain::MerchantContext,
        _customer: &Option<domain::Customer>,
        _merchant_connector_account: &domain::MerchantConnectorAccountTypeDetails,
        _merchant_recipient_data: Option<MerchantRecipientData>,
        _header_payload: Option<hyperswitch_domain_models::payments::HeaderPayload>,
    ) -> RouterResult<RouterData<frm_api::Sale, FraudCheckSaleData, FraudCheckResponseData>> {
        todo!()
    }

    #[cfg(feature = "v1")]
    async fn construct_router_data<'a>(
        &self,
        state: &SessionState,
        connector_id: &str,
        merchant_context: &domain::MerchantContext,
        customer: &Option<domain::Customer>,
        merchant_connector_account: &helpers::MerchantConnectorAccountType,
        _merchant_recipient_data: Option<MerchantRecipientData>,
        header_payload: Option<hyperswitch_domain_models::payments::HeaderPayload>,
    ) -> RouterResult<RouterData<frm_api::Sale, FraudCheckSaleData, FraudCheckResponseData>> {
        let status = storage_enums::AttemptStatus::Pending;

        let auth_type: ConnectorAuthType = merchant_connector_account
            .get_connector_account_details()
            .parse_value("ConnectorAuthType")
            .change_context(errors::ApiErrorResponse::MerchantConnectorAccountNotFound {
                id: "ConnectorAuthType".to_string(),
            })?;

        let customer_id = customer.to_owned().map(|customer| customer.customer_id);

        let router_data = RouterData {
            flow: std::marker::PhantomData,
            merchant_id: merchant_context.get_merchant_account().get_id().clone(),
            customer_id,
            connector: connector_id.to_string(),
            payment_id: self.payment_intent.payment_id.get_string_repr().to_owned(),
            attempt_id: self.payment_attempt.attempt_id.clone(),
            tenant_id: state.tenant.tenant_id.clone(),
            status,
            payment_method: self
                .payment_attempt
                .payment_method
                .ok_or(errors::ApiErrorResponse::PaymentMethodNotFound)?,
            connector_auth_type: auth_type,
            description: None,
            address: self.address.clone(),
            auth_type: storage_enums::AuthenticationType::NoThreeDs,
            connector_meta_data: None,
            connector_wallets_details: None,
            amount_captured: None,
            minor_amount_captured: None,
            request: FraudCheckSaleData {
                amount: self
                    .payment_attempt
                    .net_amount
                    .get_total_amount()
                    .get_amount_as_i64(),
                order_details: self.order_details.clone(),
                currency: self.payment_attempt.currency,
                email: customer
                    .clone()
                    .and_then(|customer_data| {
                        customer_data
                            .email
                            .map(|email| Email::try_from(email.into_inner().expose()))
                    })
                    .transpose()
                    .change_context(errors::ApiErrorResponse::InvalidDataValue {
                        field_name: "customer.customer_data.email",
                    })?,
            },
            response: Ok(FraudCheckResponseData::TransactionResponse {
                resource_id: ResponseId::ConnectorTransactionId("".to_string()),
                connector_metadata: None,
                status: storage_enums::FraudCheckStatus::Pending,
                score: None,
                reason: None,
            }),
            access_token: None,
            session_token: None,
            reference_id: None,
            payment_method_token: None,
            connector_customer: None,
            preprocessing_id: None,
            payment_method_status: None,
            connector_request_reference_id: uuid::Uuid::new_v4().to_string(),
            test_mode: None,
            recurring_mandate_payment_data: None,
            #[cfg(feature = "payouts")]
            payout_method_data: None,
            #[cfg(feature = "payouts")]
            quote_id: None,
            payment_method_balance: None,
            connector_http_status_code: None,
            external_latency: None,
            connector_api_version: None,
            apple_pay_flow: None,
            frm_metadata: self.frm_metadata.clone(),
            refund_id: None,
            dispute_id: None,
            connector_response: None,
            integrity_check: Ok(()),
            additional_merchant_data: None,
            header_payload,
            connector_mandate_request_reference_id: None,
            authentication_id: None,
            psd2_sca_exemption_type: None,
            raw_connector_response: None,
            is_payment_id_from_merchant: None,
        };

        Ok(router_data)
    }
}

#[async_trait]
impl FeatureFrm<frm_api::Sale, FraudCheckSaleData> for FrmSaleRouterData {
    async fn decide_frm_flows<'a>(
        mut self,
        state: &SessionState,
        connector: &FraudCheckConnectorData,
        call_connector_action: payments::CallConnectorAction,
        merchant_context: &domain::MerchantContext,
    ) -> RouterResult<Self> {
        decide_frm_flow(
            &mut self,
            state,
            connector,
            call_connector_action,
            merchant_context,
        )
        .await
    }
}

pub async fn decide_frm_flow(
    router_data: &mut FrmSaleRouterData,
    state: &SessionState,
    connector: &FraudCheckConnectorData,
    call_connector_action: payments::CallConnectorAction,
    _merchant_context: &domain::MerchantContext,
) -> RouterResult<FrmSaleRouterData> {
    let connector_integration: services::BoxedFrmConnectorIntegrationInterface<
        frm_api::Sale,
        FraudCheckSaleData,
        FraudCheckResponseData,
    > = connector.connector.get_connector_integration();
    let resp = services::execute_connector_processing_step(
        state,
        connector_integration,
        router_data,
        call_connector_action,
        None,
        None,
    )
    .await
    .to_payment_failed_response()?;

    Ok(resp)
}
