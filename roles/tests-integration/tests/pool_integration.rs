use integration_tests_sv2::*;

use crate::sniffer::MessageDirection;
use const_sv2::MESSAGE_TYPE_NEW_TEMPLATE;
use roles_logic_sv2::{
    common_messages_sv2::{Protocol, SetupConnection},
    parsers::{CommonMessages, PoolMessages, TemplateDistribution},
};

// This test starts a Template Provider and a Pool, and checks if they exchange the correct
// messages upon connection.
// The Sniffer is used as a proxy between the Upstream(Template Provider) and Downstream(Pool). The
// Pool will connect to the Sniffer, and the Sniffer will connect to the Template Provider.
#[tokio::test]
async fn success_pool_template_provider_connection() {
    let (_tp, tp_addr) = start_template_provider().await;
    let (sniffer, sniffer_addr) = start_sniffer("".to_string(), tp_addr, true, None).await;
    let _ = start_pool(Some(sniffer_addr)).await;
    // here we assert that the downstream(pool in this case) have sent `SetupConnection` message
    // with the correct parameters, protocol, flags, min_version and max_version.  Note that the
    // macro can take any number of arguments after the message argument, but the order is
    // important where a property should be followed by its value.
    assert_common_message!(
        &sniffer.next_message_from_downstream(),
        SetupConnection,
        protocol,
        Protocol::TemplateDistributionProtocol,
        flags,
        0,
        min_version,
        2,
        max_version,
        2
    );
    assert_common_message!(
        &sniffer.next_message_from_upstream(),
        SetupConnectionSuccess
    );
    assert_tp_message!(
        &sniffer.next_message_from_downstream(),
        CoinbaseOutputDataSize
    );
    sniffer
        .wait_for_message_type(MessageDirection::ToDownstream, MESSAGE_TYPE_NEW_TEMPLATE)
        .await;
    assert_tp_message!(&sniffer.next_message_from_upstream(), NewTemplate);
    assert_tp_message!(sniffer.next_message_from_upstream(), SetNewPrevHash);
}
