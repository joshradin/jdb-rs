use jdwp_client_tests::JavaInstance;
use std::io;
use jdwp_client::commands::Version;

#[test_log::test(tokio::test)]
async fn test_get_jvm_version() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    let version = client.send(Version).await?;
    println!("got version: {version:#?}");
    assert!(version.major >= 8, "major is not >= 8: {}", version.major);
    assert!(version.minor >= 0, "minor is not >= 0: {}", version.minor);
    Ok(())
}
