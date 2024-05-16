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
}

#[tokio::test]
async fn fetch_nonexistent_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("foo").await;
    assert!(resp.is_err());
}


#[tokio::test]
async fn fetch_user_posts() {
    let client = Threads::new().unwrap();
    let user_resp = client.fetch_user("zuck").await;
    assert!(user_resp.is_ok());

    let user = user_resp.unwrap();
    let posts = user.fetch_posts(Some(0..3)).await;
    assert!(posts.is_ok());

    let resp = posts.unwrap();
    println!("{:#?}", resp);

    assert!(resp.len() > 0);
}