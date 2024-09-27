use jdb_test_fixtures::JavaInstance;
use jdi_rs::*;
use test_log::test;

#[test(tokio::test)]
async fn test_all_classes() -> eyre::Result<()> {
    let jvm_instance = JavaInstance::new(0, "BusyBeaver").await?;
    let vm = VirtualMachineManager::attach(("127.0.0.1", jvm_instance.port())).await?;

    let all_classes = vm.all_classes().await?;

    let object_class = all_classes.into_iter()
        .find(|r| r.signature() == "Ljava/lang/Object;")
        .expect("no object class");

    println!("object_class: {object_class:#?}");

    Ok(())
}
