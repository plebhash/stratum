use const_sv2::MESSAGE_TYPE_SETUP_CONNECTION_ERROR;
use integration_tests_sv2::*;
use roles_logic_sv2::{
    common_messages_sv2::SetupConnectionError,
    parsers::{CommonMessages, PoolMessages},
};
use sniffer::{InterceptMessage, MessageDirection};
use std::convert::TryInto;

#[tokio::test]
async fn test_sniffer_intercept() {
    let (_tp, tp_addr) = start_template_provider(None).await;
    use const_sv2::MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS;
    let message_replacement =
        PoolMessages::Common(CommonMessages::SetupConnectionError(SetupConnectionError {
            flags: 0,
            error_code: "unsupported-feature-flags"
                .to_string()
                .into_bytes()
                .try_into()
                .unwrap(),
        }));
    let intercept = InterceptMessage::new(
        MessageDirection::ToDownstream,
        MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
        message_replacement,
        MESSAGE_TYPE_SETUP_CONNECTION_ERROR,
    );
    let (sniffer, sniffer_addr) =
        start_sniffer("".to_string(), tp_addr, false, Some(vec![intercept])).await;
    let _ = start_pool(Some(sniffer_addr)).await;
    assert_common_message!(&sniffer.next_message_from_downstream(), SetupConnection);
    assert_common_message!(&sniffer.next_message_from_upstream(), SetupConnectionError);
}
