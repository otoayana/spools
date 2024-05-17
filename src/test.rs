use crate::Threads;

#[tokio::test]
async fn new_client() {
    let client = Threads::new();
    assert!(client.is_ok());
}

#[tokio::test]
async fn fetch_existing_user() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_user("zuck").await;
    assert!(resp.is_ok());

    let user = resp.unwrap();
    println!("{:#?}", user);
    assert_eq!(user.id, 314216);
}

#[tokio::test]
async fn fetch_nonexistent_user() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_user("cant-have-dashes").await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn fetch_existing_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("C6EbeLPxovW").await;
    println!("{:#?}", resp);
    assert!(resp.is_ok());

    let post = resp.unwrap();
    println!("{:#?}", post);
    assert_eq!(post.id, "3351924843586423766");
    assert_eq!(post.name, "zuck");
}

#[tokio::test]
async fn fetch_nonexistent_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("foo").await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn convert_to_post() {
    let client = Threads::new().unwrap();
    let child_resp = client.fetch_post("C6brVPxR1fZ").await;
    println!("{:#?}", child_resp);
    assert!(child_resp.is_ok());

    let child_post = child_resp.unwrap();
    println!("{:#?}", child_post);

    let resp = child_post.parents[0].to_post().await;
    println!("{:#?}", resp);
    assert!(resp.is_ok());

    let post = resp.unwrap();
    assert_eq!(post.id, "3358445536292748283");
    assert_eq!(post.name, "metaquest");
}

#[tokio::test]
async fn convert_to_subpost() {
    let client = Threads::new().unwrap();
    let post_resp = client.fetch_post("C6brVPxR1fZ").await;
    println!("{:#?}", post_resp);
    assert!(post_resp.is_ok());

    let post = post_resp.unwrap();
    println!("{:#?}", post);

    let resp = post.to_subpost(None).await;
    println!("{:#?}", resp);
    assert!(resp.is_ok());

    let subpost = resp.unwrap();
    assert_eq!(subpost.code, "C6brVPxR1fZ");
}
