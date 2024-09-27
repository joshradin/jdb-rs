use jdb_test_fixtures::JavaInstance;
use jdi_rs::connect::spi::TransportService;
use jdi_rs::connect::{Connector, Transport};
use jdi_rs::{VirtualMachine, VirtualMachineManager};
use test_log::test;

#[test(tokio::test)]
async fn test_create_tcp_attaching_connector() -> eyre::Result<()> {
    let jvm_instance = JavaInstance::new(0, "BusyBeaver").await?;
    let _vm = VirtualMachineManager::attach(("localhost", jvm_instance.port()))
        .await
        .expect("could not create service");

    Ok(())
}
