use crate::Threads;

#[test]
fn new_client() {
    let client = Threads::new();
    assert!(client.is_ok());
}

#[test]
fn fetch_existing_user() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_user("zuck");
    assert!(resp.is_ok());

    let maybe_user = resp.unwrap();
    assert!(maybe_user.is_some());

    let user = maybe_user.unwrap();
    println!("{:#?}", user);
    assert_eq!(user.id, 314216);
}

#[test]
fn fetch_nonexistent_user() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_user("cant-have-dashes");
    assert!(resp.is_ok());

    let maybe_user = resp.unwrap();
    assert!(maybe_user.is_none());
}

#[test]
fn fetch_existing_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("C6EbeLPxovW");
    assert!(resp.is_ok());

    let maybe_post = resp.unwrap();
    assert!(maybe_post.is_some());

    let post = maybe_post.unwrap();
    assert_eq!(post.id, "3351924843586423766");
}

#[test]
fn fetch_nonexistent_post() {
    let client = Threads::new().unwrap();
    let resp = client.fetch_post("foo");
    assert!(resp.is_ok());

    let maybe_post = resp.unwrap();
    assert!(maybe_post.is_none());
}
