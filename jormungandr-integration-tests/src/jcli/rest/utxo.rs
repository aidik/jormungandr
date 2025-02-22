use crate::common::configuration::genesis_model::Fund;
use crate::common::jcli_wrapper;
use crate::common::startup;
use chain_addr::Discrimination;

#[test]
pub fn test_correct_utxos_are_read_from_node() {
    let sender_private_key = jcli_wrapper::assert_key_generate_default();
    println!("Sender private key generated: {}", &sender_private_key);

    let reciever_private_key = jcli_wrapper::assert_key_generate_default();
    println!("Reciever private key generated: {}", &reciever_private_key);

    let sender_public_key = jcli_wrapper::assert_key_to_public_default(&sender_private_key);
    println!("Sender public key generated: {}", &sender_public_key);

    let reciever_public_key = jcli_wrapper::assert_key_to_public_default(&reciever_private_key);
    println!("Reciever public key generated: {}", &reciever_public_key);

    let sender_address =
        jcli_wrapper::assert_address_single(&sender_public_key, Discrimination::Test);
    println!("Sender address generated: {}", &sender_address);

    let reciever_address =
        jcli_wrapper::assert_address_single(&reciever_public_key, Discrimination::Test);
    println!("Reciever address generated: {}", &reciever_address);

    let mut funds = vec![
        Fund {
            address: reciever_address.clone(),
            value: 100.into(),
        },
        Fund {
            address: sender_address.clone(),
            value: 100.into(),
        },
    ];

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(funds.clone())
        .build();
    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node(&mut config);
    let mut content = jcli_wrapper::assert_rest_utxo_get(&jormungandr_rest_address);

    funds.sort_by_key(|fund| fund.address.clone());
    content.sort_by_key(|utxo| utxo.address().to_string());
    assert_eq!(content.len(), funds.len());
    assert_eq!(funds[0].address, content[0].address().to_string());
    assert_eq!(
        funds[0].value.to_string(),
        content[0].associated_fund().to_string()
    );
    assert_eq!(funds[1].address, content[1].address().to_string());
    assert_eq!(
        funds[1].value.to_string(),
        content[1].associated_fund().to_string()
    );
}
