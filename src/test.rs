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

    let maybe_user = resp.unwrap();
    assert!(maybe_user.is_some());

    let user = maybe_user.unwrap();
    println!("{:#?}", user);
    assert_eq!(user.id, 314216);
}

#[tokio::test]
async fn fetch_nonexistent_user() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_user("cant-have-dashes").await;
    assert!(resp.is_ok());

    let maybe_user = resp.unwrap();
    assert!(maybe_user.is_none());
}

#[tokio::test]
async fn fetch_existing_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("C6EbeLPxovW").await;
    assert!(resp.is_ok());

    let maybe_post = resp.unwrap();
    assert!(maybe_post.is_some());

    let post = maybe_post.unwrap();
    assert_eq!(post.id, "3351924843586423766");
}

#[tokio::test]
async fn fetch_nonexistent_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("foo").await;
    assert!(resp.is_ok());

    let maybe_post = resp.unwrap();
    assert!(maybe_post.is_none());
}

#[tokio::test]
async fn fetch_user_posts() {
	let client = Threads::new().unwrap();
	let user_resp = client.fetch_user("zuck").await;
	assert!(user_resp.is_ok());

	let maybe_user = user_resp.unwrap();
	assert!(maybe_user.is_some());
    
	let user = maybe_user.unwrap();
	println!("{:#?}", user);
    
	let posts = user.fetch_posts(None).await;
	assert!(posts.is_ok());

	let resp = posts.unwrap();
	println!("{:#?}", resp);

	assert!(resp.len() > 0);
}

#[tokio::test]
async fn fetch_user_posts_limit() {
	let client = Threads::new().unwrap();
	let user_resp = client.fetch_user("zuck").await;
	assert!(user_resp.is_ok());

	let maybe_user = user_resp.unwrap();
	assert!(maybe_user.is_some());
    
	let user = maybe_user.unwrap();
	println!("{:#?}", user);
    
	let posts = user.fetch_posts(Some(0..3)).await;
	assert!(posts.is_ok());

	let resp = posts.unwrap();
	println!("{:#?}", resp);

	assert!(resp.len() == 3);
}

#[tokio::test]
async fn fetch_user_posts_overlimit() {
	let client = Threads::new().unwrap();
	let user_resp = client.fetch_user("zuck").await;
	assert!(user_resp.is_ok());

	let maybe_user = user_resp.unwrap();
	assert!(maybe_user.is_some());
    
	let user = maybe_user.unwrap();
	println!("{:#?}", user);
    
	let posts = user.fetch_posts(Some(28..40)).await;
	assert!(posts.is_err());
}

#[tokio::test]
async fn fetch_post_parents() {
	let client = Threads::new().unwrap();
	let post_resp = client.fetch_post("C6brVPxR1fZ").await;
	assert!(post_resp.is_ok());

	let maybe_post = post_resp.unwrap();
	assert!(maybe_post.is_some());
    
	let post = maybe_post.unwrap();
	println!("{:#?}", post);
    
	let posts = post.fetch_parents(None).await;
	assert!(posts.is_ok());

	let resp = posts.unwrap();
	println!("{:#?}", resp);

	assert!(resp[0].id == "3358445536292748283");
}

#[tokio::test]
async fn fetch_post_replies() {
	let client = Threads::new().unwrap();
	let post_resp = client.fetch_post("C6bmGvkObv7").await;
	assert!(post_resp.is_ok());

	let maybe_post = post_resp.unwrap();
	assert!(maybe_post.is_some());
    
	let post = maybe_post.unwrap();
	println!("{:#?}", post);
    
	let posts = post.fetch_replies(Some(0..1)).await;
	assert!(posts.is_ok());

	let resp = posts.unwrap();
	println!("{:#?}", resp);

	assert!(resp[0].id == "3358468523176712153");
}
