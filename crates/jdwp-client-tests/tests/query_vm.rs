use jdb_test_fixtures::JavaInstance;
use jdwp_client::commands::{AllClasses, AllThreads, ClassesBySignatures, Version};
use std::io;
use tracing::info;

#[test_log::test(tokio::test)]
async fn test_get_jvm_version() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    let version = client.send(Version).await?;
    println!("got version: {version:#?}");
    assert!(version.major >= 8, "major is not >= 8: {}", version.major);
    assert!(version.minor >= 0, "minor is not >= 0: {}", version.minor);
    client.dispose().await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_string_class() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    let data = client
        .send(ClassesBySignatures {
            signature: "Ljava/lang/String;".to_string(),
        })
        .await?;
    println!("data: {data:#?}");
    client.dispose().await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_all_classes() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    let data = client.send(AllClasses).await?;
    info!("initialized:");
    for x in data.classes {
        if x.status.initialized() {
            info!(" - {} ({:?})", x.signature, x.id);
        }
    }
    client.dispose().await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_all_threads() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    let data = client.send(AllThreads).await?;
    println!("data: {data:#?}");
    client.dispose().await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_dispose() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    client
        .on_event(|_, e| async move {
            println!("got event: {e:?}");
            Ok(())
        })
        .await;
    client.dispose().await?;
    println!("client disposed");

    Ok(())
}
