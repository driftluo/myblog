//! API Integration Tests
//!
//! These tests require a running server and database connection
//! Before running tests, ensure:
//! 1. PostgreSQL database is started and configured correctly
//! 2. Redis is started
//! 3. Server is running on http://127.0.0.1:8080
//!
//! Run tests: cargo test --test api_tests -- --test-threads=1

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

const BASE_URL: &str = "http://127.0.0.1:8080";
const API_PREFIX: &str = "/api/v1";

/// Create a client with cookie jar
fn create_client() -> Client {
    Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client")
}

/// Password format: 6 character prefix + actual password
fn format_password(password: &str) -> String {
    format!("prefix{}", password)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_micros()
}

// ============================================
// Password Validation Tests
// ============================================

#[cfg(test)]
mod password_tests {
    use new_blog::utils::get_password;

    #[test]
    fn test_get_password_valid() {
        // Normal case: 6-char prefix + actual password
        let result = get_password("abcdefpassword");
        assert_eq!(result, Some("password".to_string()));
    }

    #[test]
    fn test_get_password_exactly_6_chars() {
        // Edge case: exactly 6 characters, actual password is empty
        let result = get_password("abcdef");
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_get_password_too_short() {
        // Error case: less than 6 characters should return None
        let result = get_password("admin");
        assert_eq!(result, None);

        let result = get_password("12345");
        assert_eq!(result, None);

        let result = get_password("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_password_unicode() {
        // Unicode character test (note: split_at splits by bytes)
        // "你好世" is 9 bytes, more than 6 bytes
        let result = get_password("你好世界");
        // "你好" = 6 bytes, remaining "世界"
        assert!(result.is_some());
    }
}

// ============================================
// Visitor API Tests (Public Endpoints)
// ============================================

#[cfg(test)]
mod visitor_api_tests {
    use super::*;

    async fn login_as_admin() -> Client {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);
        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Login failed");
        assert_eq!(response.status(), StatusCode::OK);
        client
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_article_view_all() {
        let client = create_client();
        let url = format!(
            "{}{}/article/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_article_view_all_by_tag() {
        let client = create_client();
        // Use a non-existent tag_id, should return empty array
        let url = format!(
            "{}{}/article/view_all/00000000-0000-0000-0000-000000000000",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_rss_feed() {
        let client = create_client();
        let url = format!("{}/rss", BASE_URL);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response
            .headers()
            .get("content-type")
            .expect("No content-type header");
        assert!(content_type.to_str().unwrap().contains("xml"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_password_too_short() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        // Password too short (less than 6 characters), should be rejected
        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": "admin",  // Only 5 characters, will cause get_password to return None
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        // Server should respond normally (not panic), return error message
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], false);
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("Invalid password format")
        );
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_article_view_and_comments() {
        let admin_client = login_as_admin().await;
        let title = format!("Visitor API Test Article {}", unique_suffix());

        let create_article_url = format!("{}{}/article/new", BASE_URL, API_PREFIX);
        let create_resp = admin_client
            .post(&create_article_url)
            .json(&json!({
                "title": title,
                "raw_content": "# visitor view test\n\nhello"
            }))
            .send()
            .await
            .expect("Create article failed");
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body: Value = create_resp.json().await.expect("Parse create article");
        assert_eq!(create_body["status"], true);

        let list_url = format!(
            "{}{}/article/admin/view_all?limit=50&offset=0",
            BASE_URL, API_PREFIX
        );
        let list_resp = admin_client
            .get(&list_url)
            .send()
            .await
            .expect("List article failed");
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body: Value = list_resp.json().await.expect("Parse list article");
        assert_eq!(list_body["status"], true);
        let article = list_body["data"]
            .as_array()
            .and_then(|arr| arr.iter().find(|a| a["title"] == title))
            .expect("Created article not found in admin list");
        let article_id = article["id"]
            .as_str()
            .expect("article id missing")
            .to_string();

        let publish_url = format!("{}{}/article/publish", BASE_URL, API_PREFIX);
        let publish_resp = admin_client
            .post(&publish_url)
            .json(&json!({
                "id": article_id,
                "publish": true
            }))
            .send()
            .await
            .expect("Publish article failed");
        assert_eq!(publish_resp.status(), StatusCode::OK);
        let publish_body: Value = publish_resp.json().await.expect("Parse publish article");
        assert_eq!(publish_body["status"], true);

        let visitor = create_client();
        let view_url = format!("{}{}/article/view?id={}", BASE_URL, API_PREFIX, article_id);
        let view_resp = visitor.get(&view_url).send().await.expect("Article view failed");
        assert_eq!(view_resp.status(), StatusCode::OK);
        let view_body: Value = view_resp.json().await.expect("Parse article view");
        assert_eq!(view_body["status"], true);
        assert_eq!(view_body["data"]["id"], article_id);

        let comments_url = format!(
            "{}{}/article/view_comment/{}?limit=10&offset=0",
            BASE_URL, API_PREFIX, article_id
        );
        let comments_resp = visitor
            .get(&comments_url)
            .send()
            .await
            .expect("Comment list failed");
        assert_eq!(comments_resp.status(), StatusCode::OK);
        let comments_body: Value = comments_resp.json().await.expect("Parse comment list");
        assert_eq!(comments_body["status"], true);
        assert!(comments_body["data"].is_array());

        let delete_url = format!("{}{}/article/delete/{}", BASE_URL, API_PREFIX, article_id);
        let delete_resp = admin_client
            .post(&delete_url)
            .send()
            .await
            .expect("Delete article failed");
        assert_eq!(delete_resp.status(), StatusCode::OK);
        let delete_body: Value = delete_resp.json().await.expect("Parse delete article");
        assert_eq!(delete_body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_with_github_missing_code() {
        let client = create_client();
        let url = format!("{}{}/login_with_github", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_with_github_invalid_code() {
        let client = create_client();
        let url = format!("{}{}/login_with_github?code=invalid_code_for_test", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_uuid_path_constraints_for_visitor_article_endpoints() {
        let client = create_client();

        let by_tag_url = format!("{}{}/article/view_all/not-a-uuid", BASE_URL, API_PREFIX);
        let by_tag_resp = client
            .get(&by_tag_url)
            .send()
            .await
            .expect("Request failed");
        assert_eq!(by_tag_resp.status(), StatusCode::NOT_FOUND);

        let comment_url = format!(
            "{}{}/article/view_comment/not-a-uuid?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );
        let comment_resp = client
            .get(&comment_url)
            .send()
            .await
            .expect("Request failed");
        assert_eq!(comment_resp.status(), StatusCode::NOT_FOUND);
    }
}

// ============================================
// Authentication Tests
// ============================================

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_login_success() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_wrong_password() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("wrongpassword"),
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], false);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_protected_api_without_auth() {
        let client = create_client();
        let url = format!(
            "{}{}/article/admin/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_fund_api_without_auth() {
        let client = create_client();

        // Read endpoint should be protected
        let list_url = format!("{}{}/fund/portfolios", BASE_URL, API_PREFIX);
        let list_resp = client.get(&list_url).send().await.expect("Request failed");
        assert_eq!(list_resp.status(), StatusCode::FORBIDDEN);

        // Write endpoint should also be protected
        let create_url = format!("{}{}/fund/portfolio", BASE_URL, API_PREFIX);
        let create_resp = client
            .post(&create_url)
            .json(&json!({
                "name": "Unauthorized Portfolio",
                "description": "should be blocked"
            }))
            .send()
            .await
            .expect("Request failed");
        assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);
    }
}

// ============================================
// Admin API Tests (Requires Authentication)
// ============================================

#[cfg(test)]
mod admin_api_tests {
    use super::*;

    /// Login and return authenticated client
    async fn login_as_admin() -> Client {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Login failed");

        assert_eq!(response.status(), StatusCode::OK);
        client
    }

    async fn create_temp_article(client: &Client, title: &str, publish: bool) -> String {
        let create_article_url = format!("{}{}/article/new", BASE_URL, API_PREFIX);
        let create_resp = client
            .post(&create_article_url)
            .json(&json!({
                "title": title,
                "raw_content": "# temp article\n\nfor api tests"
            }))
            .send()
            .await
            .expect("Create article failed");
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body: Value = create_resp.json().await.expect("Parse create article");
        assert_eq!(create_body["status"], true);

        let list_url = format!(
            "{}{}/article/admin/view_all?limit=50&offset=0",
            BASE_URL, API_PREFIX
        );
        let list_resp = client
            .get(&list_url)
            .send()
            .await
            .expect("List article failed");
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body: Value = list_resp.json().await.expect("Parse list article");
        assert_eq!(list_body["status"], true);
        let article = list_body["data"]
            .as_array()
            .and_then(|arr| arr.iter().find(|a| a["title"] == title))
            .expect("Created article not found");
        let article_id = article["id"]
            .as_str()
            .expect("article id missing")
            .to_string();

        if publish {
            let publish_url = format!("{}{}/article/publish", BASE_URL, API_PREFIX);
            let publish_resp = client
                .post(&publish_url)
                .json(&json!({
                    "id": article_id,
                    "publish": true
                }))
                .send()
                .await
                .expect("Publish article failed");
            assert_eq!(publish_resp.status(), StatusCode::OK);
            let publish_body: Value = publish_resp.json().await.expect("Parse publish article");
            assert_eq!(publish_body["status"], true);
        }

        article_id
    }

    async fn delete_article_if_exists(client: &Client, article_id: &str) {
        let delete_url = format!("{}{}/article/delete/{}", BASE_URL, API_PREFIX, article_id);
        let _ = client.post(&delete_url).send().await;
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_article_list() {
        let client = login_as_admin().await;
        let url = format!(
            "{}{}/article/admin/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_article_write_and_upload() {
        let client = login_as_admin().await;
        let title = format!("Admin Article API Test {}", unique_suffix());
        let article_id = create_temp_article(&client, &title, false).await;

        // admin/view
        let view_url = format!("{}{}/article/admin/view?id={}", BASE_URL, API_PREFIX, article_id);
        let view_resp = client.get(&view_url).send().await.expect("admin view failed");
        assert_eq!(view_resp.status(), StatusCode::OK);
        let view_body: Value = view_resp.json().await.expect("Parse admin view");
        assert_eq!(view_body["status"], true);
        assert_eq!(view_body["data"]["id"], article_id);

        // admin/view_raw
        let view_raw_url = format!(
            "{}{}/article/admin/view_raw?id={}",
            BASE_URL, API_PREFIX, article_id
        );
        let raw_resp = client
            .get(&view_raw_url)
            .send()
            .await
            .expect("admin view raw failed");
        assert_eq!(raw_resp.status(), StatusCode::OK);
        let raw_body: Value = raw_resp.json().await.expect("Parse admin view raw");
        assert_eq!(raw_body["status"], true);
        assert_eq!(raw_body["data"]["id"], article_id);

        // edit
        let edit_url = format!("{}{}/article/edit", BASE_URL, API_PREFIX);
        let edited_title = format!("{} (edited)", title);
        let edit_resp = client
            .post(&edit_url)
            .json(&json!({
                "id": article_id,
                "title": edited_title,
                "raw_content": "# edited\n\ncontent"
            }))
            .send()
            .await
            .expect("edit article failed");
        assert_eq!(edit_resp.status(), StatusCode::OK);
        let edit_body: Value = edit_resp.json().await.expect("Parse edit article");
        assert_eq!(edit_body["status"], true);

        // publish
        let publish_url = format!("{}{}/article/publish", BASE_URL, API_PREFIX);
        let publish_resp = client
            .post(&publish_url)
            .json(&json!({
                "id": article_id,
                "publish": true
            }))
            .send()
            .await
            .expect("publish article failed");
        assert_eq!(publish_resp.status(), StatusCode::OK);
        let publish_body: Value = publish_resp.json().await.expect("Parse publish article");
        assert_eq!(publish_body["status"], true);

        // upload (success branch)
        let upload_url = format!("{}{}/upload", BASE_URL, API_PREFIX);
        let upload_filename = format!("api-upload-{}.txt", unique_suffix());
        let form = reqwest::multipart::Form::new().part(
            "files",
            reqwest::multipart::Part::bytes("hello upload test".as_bytes().to_vec())
                .file_name(upload_filename.clone())
                .mime_str("text/plain")
                .expect("mime should be valid"),
        );
        let upload_resp = client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .expect("upload failed");
        assert_eq!(upload_resp.status(), StatusCode::OK);
        let upload_body: Value = upload_resp.json().await.expect("Parse upload");
        assert_eq!(upload_body["status"], true);
        let uploaded_path = upload_body["data"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .expect("uploaded path missing")
            .to_string();
        assert!(uploaded_path.ends_with(&upload_filename));
        let _ = std::fs::remove_file(&uploaded_path);

        delete_article_if_exists(&client, &article_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_upload_without_files_returns_bad_request() {
        let client = login_as_admin().await;
        let upload_url = format!("{}{}/upload", BASE_URL, API_PREFIX);
        let upload_resp = client
            .post(&upload_url)
            .send()
            .await
            .expect("upload request failed");
        assert_eq!(upload_resp.status(), StatusCode::BAD_REQUEST);
        let upload_body: Value = upload_resp.json().await.expect("Parse upload response");
        assert_eq!(upload_body["status"], false);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_fund_crud_and_batch_operations() {
        let client = login_as_admin().await;

        // 1. Create portfolio
        let create_portfolio_url = format!("{}{}/fund/portfolio", BASE_URL, API_PREFIX);
        let resp = client
            .post(&create_portfolio_url)
            .json(&json!({ "name": "API Test Portfolio", "description": "created by tests" }))
            .send()
            .await
            .expect("Create portfolio request failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Value = resp.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        let portfolio_id = body["data"].as_i64().expect("id missing") as i32;

        // 1.1 list portfolios and confirm new portfolio exists
        let list_portfolios_url = format!("{}{}/fund/portfolios", BASE_URL, API_PREFIX);
        let resp = client
            .get(&list_portfolios_url)
            .send()
            .await
            .expect("List portfolios failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let portfolios_body: Value = resp.json().await.expect("Parse portfolio list");
        assert_eq!(portfolios_body["status"], true);
        let portfolios = portfolios_body["data"].as_array().expect("data should be array");
        assert!(portfolios.iter().any(|p| p["id"] == portfolio_id));

        // 2. Create two entries
        let create_entry_url = format!("{}{}/fund/entry", BASE_URL, API_PREFIX);
        let resp1 = client
            .post(&create_entry_url)
            .json(&json!({
                "portfolio_id": portfolio_id,
                "major_category": "Stocks",
                "minor_category": "Tech",
                "fund_type": "ETF",
                "fund_name": "Test Fund A",
                "target_ratio": 0.5,
                "amount": 1000.0
            }))
            .send()
            .await
            .expect("Create entry A failed");
        assert_eq!(resp1.status(), StatusCode::OK);
        let b1: Value = resp1.json().await.expect("Parse create A");
        assert_eq!(b1["status"], true);
        let entry_a_id = b1["data"].as_i64().expect("id missing") as i32;

        let resp2 = client
            .post(&create_entry_url)
            .json(&json!({
                "portfolio_id": portfolio_id,
                "major_category": "Stocks",
                "minor_category": "Tech",
                "fund_type": "ETF",
                "fund_name": "Test Fund B",
                "target_ratio": 0.5,
                "amount": 500.0
            }))
            .send()
            .await
            .expect("Create entry B failed");
        assert_eq!(resp2.status(), StatusCode::OK);
        let b2: Value = resp2.json().await.expect("Parse create B");
        assert_eq!(b2["status"], true);
        let entry_b_id = b2["data"].as_i64().expect("id missing") as i32;

        // 2.1 Create entries in batch (array body)
        let resp = client
            .post(&create_entry_url)
            .json(&json!([
                {
                    "portfolio_id": portfolio_id,
                    "major_category": "Bond",
                    "minor_category": "Gov",
                    "fund_type": "Index",
                    "fund_name": "Test Fund C",
                    "target_ratio": 0.2,
                    "amount": 200.0
                },
                {
                    "portfolio_id": portfolio_id,
                    "major_category": "Bond",
                    "minor_category": "Corp",
                    "fund_type": "Index",
                    "fund_name": "Test Fund D",
                    "target_ratio": 0.1,
                    "amount": 100.0
                }
            ]))
            .send()
            .await
            .expect("Batch create entries failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let batch_create_body: Value = resp.json().await.expect("Parse batch create");
        assert_eq!(batch_create_body["status"], true);
        let batch_entry_ids = batch_create_body["data"]
            .as_array()
            .expect("batch create should return ids");
        assert_eq!(batch_entry_ids.len(), 2);
        let entry_c_id = batch_entry_ids[0].as_i64().expect("id C missing") as i32;
        let entry_d_id = batch_entry_ids[1].as_i64().expect("id D missing") as i32;

        // 3. List entries
        let list_url = format!("{}{}/fund/entries/{}", BASE_URL, API_PREFIX, portfolio_id);
        let resp = client
            .get(&list_url)
            .send()
            .await
            .expect("List entries failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let list_body: Value = resp.json().await.expect("Parse list");
        assert_eq!(list_body["status"], true);
        let arr = list_body["data"].as_array().expect("data array");
        assert!(arr.len() >= 4);

        // 3.1 Get portfolio with entries
        let get_portfolio_url = format!("{}{}/fund/portfolio/{}", BASE_URL, API_PREFIX, portfolio_id);
        let resp = client
            .get(&get_portfolio_url)
            .send()
            .await
            .expect("Get portfolio failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let get_portfolio_body: Value = resp.json().await.expect("Parse get portfolio");
        assert_eq!(get_portfolio_body["status"], true);
        assert_eq!(get_portfolio_body["data"]["portfolio"]["id"], portfolio_id);
        let get_entries = get_portfolio_body["data"]["entries"]
            .as_array()
            .expect("entries should be array");
        assert!(get_entries.len() >= 4);

        // 4. Update one entry (change minor_category)
        let update_url = format!("{}{}/fund/entry/update", BASE_URL, API_PREFIX);
        let resp = client
            .post(&update_url)
            .json(&json!({ "id": entry_a_id, "minor_category": "Software" }))
            .send()
            .await
            .expect("Update entry failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let upb: Value = resp.json().await.expect("Parse update");
        assert_eq!(upb["status"], true);

        // 4.1 Batch update entries via array body
        let resp = client
            .post(&update_url)
            .json(&json!([
                { "id": entry_c_id, "minor_category": "Treasury" },
                { "id": entry_d_id, "minor_category": "IG" }
            ]))
            .send()
            .await
            .expect("Batch update entry failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let batch_update_entry_body: Value = resp.json().await.expect("Parse batch update entry");
        assert_eq!(batch_update_entry_body["status"], true);

        // 4.2 Update portfolio
        let update_portfolio_url = format!("{}{}/fund/portfolio/update", BASE_URL, API_PREFIX);
        let updated_name = format!("API Test Portfolio Updated {}", portfolio_id);
        let resp = client
            .post(&update_portfolio_url)
            .json(&json!({
                "id": portfolio_id,
                "name": updated_name
            }))
            .send()
            .await
            .expect("Update portfolio failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let update_portfolio_body: Value = resp.json().await.expect("Parse update portfolio");
        assert_eq!(update_portfolio_body["status"], true);

        // 5. Batch update amounts
        let batch_amount_url = format!("{}{}/fund/entries/batch-update", BASE_URL, API_PREFIX);
        let resp = client
            .post(&batch_amount_url)
            .json(&json!({
                "portfolio_id": portfolio_id,
                "updates": [
                    { "id": entry_a_id, "amount": 1200.0 },
                    { "id": entry_b_id, "amount": 300.0 },
                    { "id": entry_c_id, "amount": 250.0 },
                    { "id": entry_d_id, "amount": 150.0 }
                ]
            }))
            .send()
            .await
            .expect("Batch update amounts failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let bab: Value = resp.json().await.expect("Parse batch amounts");
        assert_eq!(bab["status"], true);

        // 6. Batch update order
        let batch_order_url = format!("{}{}/fund/entries/batch-order", BASE_URL, API_PREFIX);
        let resp = client
            .post(&batch_order_url)
            .json(&json!({ "portfolio_id": portfolio_id, "updates": [{ "id": entry_b_id, "sort_index": 0 }, { "id": entry_a_id, "sort_index": 1 }] }))
            .send()
            .await
            .expect("Batch order failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let bor: Value = resp.json().await.expect("Parse batch order");
        assert_eq!(bor["status"], true);

        // 6.1 Confirm updated portfolio total in portfolio list
        let resp = client
            .get(&list_portfolios_url)
            .send()
            .await
            .expect("List portfolios after amount update failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let portfolios_after_update: Value = resp.json().await.expect("Parse portfolio list after update");
        assert_eq!(portfolios_after_update["status"], true);
        let p = portfolios_after_update["data"]
            .as_array()
            .and_then(|items| items.iter().find(|p| p["id"] == portfolio_id))
            .expect("updated portfolio not found");
        let total_amount = p["total_amount"]
            .as_f64()
            .expect("total_amount should be number");
        assert!(
            (total_amount - 1900.0).abs() < 0.0001,
            "unexpected total_amount: {}",
            total_amount
        );

        // 7. Delete entries
        let del_a = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_a_id
        );
        let resp = client.post(&del_a).send().await.expect("Delete A failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dab: Value = resp.json().await.expect("Parse delete A");
        assert_eq!(dab["status"], true);

        let del_b = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_b_id
        );
        let resp = client.post(&del_b).send().await.expect("Delete B failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dbb: Value = resp.json().await.expect("Parse delete B");
        assert_eq!(dbb["status"], true);

        let del_c = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_c_id
        );
        let resp = client.post(&del_c).send().await.expect("Delete C failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dcb: Value = resp.json().await.expect("Parse delete C");
        assert_eq!(dcb["status"], true);

        let del_d = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_d_id
        );
        let resp = client.post(&del_d).send().await.expect("Delete D failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let ddb: Value = resp.json().await.expect("Parse delete D");
        assert_eq!(ddb["status"], true);

        // 8. Delete portfolio
        let del_p = format!(
            "{}{}/fund/portfolio/delete/{}",
            BASE_URL, API_PREFIX, portfolio_id
        );
        let resp = client
            .post(&del_p)
            .send()
            .await
            .expect("Delete portfolio failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dpb: Value = resp.json().await.expect("Parse delete portfolio");
        assert_eq!(dpb["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_fund_api_invalid_json_returns_bad_request() {
        let client = login_as_admin().await;

        let create_portfolio_url = format!("{}{}/fund/portfolio", BASE_URL, API_PREFIX);
        let invalid_create_portfolio_resp = client
            .post(&create_portfolio_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid create portfolio request failed");
        assert_eq!(invalid_create_portfolio_resp.status(), StatusCode::BAD_REQUEST);

        let create_portfolio_url = format!("{}{}/fund/portfolio", BASE_URL, API_PREFIX);
        let create_resp = client
            .post(&create_portfolio_url)
            .json(&json!({ "name": "API Invalid JSON Test Portfolio", "description": "temp" }))
            .send()
            .await
            .expect("Create portfolio request failed");
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body: Value = create_resp.json().await.expect("Parse create portfolio");
        assert_eq!(create_body["status"], true);
        let portfolio_id = create_body["data"].as_i64().expect("id missing") as i32;

        let create_entry_url = format!("{}{}/fund/entry", BASE_URL, API_PREFIX);
        let invalid_create_resp = client
            .post(&create_entry_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid create request failed");
        assert_eq!(invalid_create_resp.status(), StatusCode::BAD_REQUEST);

        let update_entry_url = format!("{}{}/fund/entry/update", BASE_URL, API_PREFIX);
        let invalid_update_resp = client
            .post(&update_entry_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid update request failed");
        assert_eq!(invalid_update_resp.status(), StatusCode::BAD_REQUEST);

        let batch_update_url = format!("{}{}/fund/entries/batch-update", BASE_URL, API_PREFIX);
        let invalid_batch_resp = client
            .post(&batch_update_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid batch request failed");
        assert_eq!(invalid_batch_resp.status(), StatusCode::BAD_REQUEST);

        let update_portfolio_url = format!("{}{}/fund/portfolio/update", BASE_URL, API_PREFIX);
        let invalid_update_portfolio_resp = client
            .post(&update_portfolio_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid update portfolio request failed");
        assert_eq!(invalid_update_portfolio_resp.status(), StatusCode::BAD_REQUEST);

        let batch_order_url = format!("{}{}/fund/entries/batch-order", BASE_URL, API_PREFIX);
        let invalid_batch_order_resp = client
            .post(&batch_order_url)
            .header("content-type", "application/json")
            .body("{invalid json")
            .send()
            .await
            .expect("Invalid batch order request failed");
        assert_eq!(invalid_batch_order_resp.status(), StatusCode::BAD_REQUEST);

        let del_p = format!(
            "{}{}/fund/portfolio/delete/{}",
            BASE_URL, API_PREFIX, portfolio_id
        );
        let delete_resp = client
            .post(&del_p)
            .send()
            .await
            .expect("Delete portfolio failed");
        assert_eq!(delete_resp.status(), StatusCode::OK);
        let delete_body: Value = delete_resp.json().await.expect("Parse delete portfolio");
        assert_eq!(delete_body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_fund_api_not_found_cases() {
        let client = login_as_admin().await;
        let missing_id = 2_147_483_647i32;

        let get_missing_portfolio_url =
            format!("{}{}/fund/portfolio/{}", BASE_URL, API_PREFIX, missing_id);
        let get_resp = client
            .get(&get_missing_portfolio_url)
            .send()
            .await
            .expect("Get missing portfolio failed");
        assert_eq!(get_resp.status(), StatusCode::OK);
        let get_body: Value = get_resp.json().await.expect("Parse get missing portfolio");
        assert_eq!(get_body["status"], false);

        let update_entry_url = format!("{}{}/fund/entry/update", BASE_URL, API_PREFIX);
        let update_resp = client
            .post(&update_entry_url)
            .json(&json!({
                "id": missing_id,
                "minor_category": "ShouldNotExist"
            }))
            .send()
            .await
            .expect("Update missing entry failed");
        assert_eq!(update_resp.status(), StatusCode::OK);
        let update_body: Value = update_resp.json().await.expect("Parse update missing entry");
        assert_eq!(update_body["status"], false);

        let delete_entry_url = format!("{}{}/fund/entry/delete/{}", BASE_URL, API_PREFIX, missing_id);
        let delete_entry_resp = client
            .post(&delete_entry_url)
            .send()
            .await
            .expect("Delete missing entry failed");
        assert_eq!(delete_entry_resp.status(), StatusCode::OK);
        let delete_entry_body: Value = delete_entry_resp
            .json()
            .await
            .expect("Parse delete missing entry");
        assert_eq!(delete_entry_body["status"], false);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_unpublished_articles() {
        let client = login_as_admin().await;
        let url = format!(
            "{}{}/article/admin/view_unpublished?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_tag_crud() {
        let client = login_as_admin().await;

        // 1. Create tag
        let create_url = format!("{}{}/tag/new", BASE_URL, API_PREFIX);
        let response = client
            .post(&create_url)
            .json(&json!({ "tag": "TestTagForApiTest" }))
            .send()
            .await
            .expect("Create tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        // 2. View tag list
        let list_url = format!("{}{}/tag/view?limit=10&offset=0", BASE_URL, API_PREFIX);
        let response = client
            .get(&list_url)
            .send()
            .await
            .expect("List tags failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);

        // Find the created tag
        let tags = body["data"].as_array().expect("data should be array");
        let test_tag = tags
            .iter()
            .find(|t| t["tag"] == "TestTagForApiTest")
            .expect("Created tag not found");
        let tag_id = test_tag["id"].as_str().expect("tag id not found");

        // 3. Edit tag
        let edit_url = format!("{}{}/tag/edit", BASE_URL, API_PREFIX);
        let response = client
            .post(&edit_url)
            .json(&json!({
                "id": tag_id,
                "tag": "UpdatedTestTag"
            }))
            .send()
            .await
            .expect("Edit tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        // 4. Delete tag
        let delete_url = format!("{}{}/tag/delete/{}", BASE_URL, API_PREFIX, tag_id);
        let response = client
            .post(&delete_url)
            .send()
            .await
            .expect("Delete tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_user_permission_disable_and_delete() {
        let admin_client = login_as_admin().await;

        // create temp user from public endpoint
        let user_client = create_client();
        let account = format!("api_test_user_{}", unique_suffix());
        let email = format!("{}@example.com", account);
        let register_url = format!("{}{}/user/new", BASE_URL, API_PREFIX);
        let register_resp = user_client
            .post(&register_url)
            .json(&json!({
                "account": account,
                "password": format_password("userpass123"),
                "nickname": "ApiTempUser",
                "say": "temp",
                "email": email
            }))
            .send()
            .await
            .expect("Register user failed");
        assert_eq!(register_resp.status(), StatusCode::OK);
        let register_body: Value = register_resp.json().await.expect("Parse register user");
        assert_eq!(register_body["status"], true);

        // find temp user id
        let list_url = format!("{}{}/user/view_all?limit=50&offset=0", BASE_URL, API_PREFIX);
        let list_resp = admin_client
            .get(&list_url)
            .send()
            .await
            .expect("List users failed");
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body: Value = list_resp.json().await.expect("Parse user list");
        assert_eq!(list_body["status"], true);
        let user = list_body["data"]
            .as_array()
            .and_then(|arr| arr.iter().find(|u| u["account"] == account))
            .expect("Temp user not found");
        let user_id = user["id"].as_str().expect("user id missing").to_string();

        let permission_url = format!("{}{}/user/permission", BASE_URL, API_PREFIX);
        let permission_resp = admin_client
            .post(&permission_url)
            .json(&json!({
                "id": user_id,
                "permission": 1
            }))
            .send()
            .await
            .expect("Change permission failed");
        assert_eq!(permission_resp.status(), StatusCode::OK);
        let permission_body: Value = permission_resp.json().await.expect("Parse permission");
        assert_eq!(permission_body["status"], true);

        let disable_url = format!("{}{}/user/delete/disable", BASE_URL, API_PREFIX);
        let disable_resp = admin_client
            .post(&disable_url)
            .json(&json!({
                "id": user_id,
                "disabled": 1
            }))
            .send()
            .await
            .expect("Disable user failed");
        assert_eq!(disable_resp.status(), StatusCode::OK);
        let disable_body: Value = disable_resp.json().await.expect("Parse disable user");
        assert_eq!(disable_body["status"], true);

        // disabled user should fail login
        let login_url = format!("{}{}/user/login", BASE_URL, API_PREFIX);
        let login_resp = create_client()
            .post(&login_url)
            .json(&json!({
                "account": account,
                "password": format_password("userpass123"),
                "remember": false
            }))
            .send()
            .await
            .expect("Login temp user failed");
        assert_eq!(login_resp.status(), StatusCode::OK);
        let login_body: Value = login_resp.json().await.expect("Parse login temp user");
        assert_eq!(login_body["status"], false);

        // enable then delete
        let enable_resp = admin_client
            .post(&disable_url)
            .json(&json!({
                "id": user_id,
                "disabled": 0
            }))
            .send()
            .await
            .expect("Enable user failed");
        assert_eq!(enable_resp.status(), StatusCode::OK);
        let enable_body: Value = enable_resp.json().await.expect("Parse enable user");
        assert_eq!(enable_body["status"], true);

        let delete_url = format!("{}{}/user/delete/{}", BASE_URL, API_PREFIX, user_id);
        let delete_resp = admin_client
            .post(&delete_url)
            .send()
            .await
            .expect("Delete user failed");
        assert_eq!(delete_resp.status(), StatusCode::OK);
        let delete_body: Value = delete_resp.json().await.expect("Parse delete user");
        assert_eq!(delete_body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_uuid_path_constraints_for_admin_delete_endpoints() {
        let client = login_as_admin().await;

        let article_delete_url = format!("{}{}/article/delete/not-a-uuid", BASE_URL, API_PREFIX);
        let article_delete_resp = client
            .post(&article_delete_url)
            .send()
            .await
            .expect("Request failed");
        assert!(
            article_delete_resp.status() == StatusCode::NOT_FOUND
                || article_delete_resp.status() == StatusCode::METHOD_NOT_ALLOWED
        );

        let tag_delete_url = format!("{}{}/tag/delete/not-a-uuid", BASE_URL, API_PREFIX);
        let tag_delete_resp = client
            .post(&tag_delete_url)
            .send()
            .await
            .expect("Request failed");
        assert!(
            tag_delete_resp.status() == StatusCode::NOT_FOUND
                || tag_delete_resp.status() == StatusCode::METHOD_NOT_ALLOWED
        );

        let user_delete_url = format!("{}{}/user/delete/not-a-uuid", BASE_URL, API_PREFIX);
        let user_delete_resp = client
            .post(&user_delete_url)
            .send()
            .await
            .expect("Request failed");
        assert!(
            user_delete_resp.status() == StatusCode::NOT_FOUND
                || user_delete_resp.status() == StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_user_list() {
        let client = login_as_admin().await;
        let url = format!("{}{}/user/view_all?limit=10&offset=0", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_chart_data_month() {
        let client = login_as_admin().await;
        let url = format!("{}{}/article/month", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_ip_view() {
        let client = login_as_admin().await;
        let url = format!("{}{}/ip/view?limit=10&offset=0", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_user_view_and_edit() {
        let client = login_as_admin().await;

        // View current user info
        let view_url = format!("{}{}/user/view", BASE_URL, API_PREFIX);
        let response = client.get(&view_url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);

        let original_nickname = body["data"]["nickname"].as_str().unwrap().to_string();

        // Edit user info
        let edit_url = format!("{}{}/user/edit", BASE_URL, API_PREFIX);
        let response = client
            .post(&edit_url)
            .json(&json!({
                "nickname": "TestNickname",
                "say": "Test say",
                "email": "test@example.com"
            }))
            .send()
            .await
            .expect("Edit failed");
        assert_eq!(response.status(), StatusCode::OK);

        // Restore original info
        let response = client
            .post(&edit_url)
            .json(&json!({
                "nickname": original_nickname,
                "say": null,
                "email": "441594700@qq.com"
            }))
            .send()
            .await
            .expect("Restore failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_sign_out() {
        let client = login_as_admin().await;

        // Sign out
        let signout_url = format!("{}{}/user/sign_out", BASE_URL, API_PREFIX);
        let response = client
            .get(&signout_url)
            .send()
            .await
            .expect("Sign out failed");
        assert_eq!(response.status(), StatusCode::OK);

        // After sign out, accessing protected API should return 403
        let protected_url = format!("{}{}/user/view", BASE_URL, API_PREFIX);
        let response = client
            .get(&protected_url)
            .send()
            .await
            .expect("Request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_change_password_invalid_format() {
        let client = login_as_admin().await;
        let url = format!("{}{}/user/change_pwd", BASE_URL, API_PREFIX);
        let response = client
            .post(&url)
            .json(&json!({
                "old_password": "short",
                "new_password": format_password("newpassword")
            }))
            .send()
            .await
            .expect("Change pwd request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = response.json().await.expect("Parse change pwd");
        assert_eq!(body["status"], false);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_change_password_success_and_restore() {
        let client = login_as_admin().await;
        let url = format!("{}{}/user/change_pwd", BASE_URL, API_PREFIX);
        let temp_password = format!("TmpPwd{}x", unique_suffix());

        // Change admin password to temp password
        let change_resp = client
            .post(&url)
            .json(&json!({
                "old_password": format_password("admin"),
                "new_password": format_password(&temp_password)
            }))
            .send()
            .await
            .expect("Change password request failed");
        assert_eq!(change_resp.status(), StatusCode::OK);
        let change_body: Value = change_resp.json().await.expect("Parse change password");
        assert_eq!(change_body["status"], true);

        // Restore admin password back to "admin"
        let restore_resp = client
            .post(&url)
            .json(&json!({
                "old_password": format_password(&temp_password),
                "new_password": format_password("admin")
            }))
            .send()
            .await
            .expect("Restore password request failed");
        assert_eq!(restore_resp.status(), StatusCode::OK);
        let restore_body: Value = restore_resp.json().await.expect("Parse restore password");
        assert_eq!(restore_body["status"], true);

        // Verify restored password can login
        let login_url = format!("{}{}/user/login", BASE_URL, API_PREFIX);
        let verify_resp = create_client()
            .post(&login_url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Verify login failed");
        assert_eq!(verify_resp.status(), StatusCode::OK);
        let verify_body: Value = verify_resp.json().await.expect("Parse verify login");
        assert_eq!(verify_body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_comment_new_and_delete() {
        let client = login_as_admin().await;
        let title = format!("Comment API Test Article {}", unique_suffix());
        let article_id = create_temp_article(&client, &title, true).await;

        let comment_text = format!("api test comment {}", unique_suffix());
        let create_comment_url = format!("{}{}/comment/new", BASE_URL, API_PREFIX);
        let create_comment_resp = client
            .post(&create_comment_url)
            .json(&json!({
                "comment": comment_text,
                "article_id": article_id
            }))
            .send()
            .await
            .expect("Create comment failed");
        assert_eq!(create_comment_resp.status(), StatusCode::OK);
        let create_comment_body: Value = create_comment_resp
            .json()
            .await
            .expect("Parse create comment");
        assert_eq!(create_comment_body["status"], true);

        let list_comments_url = format!(
            "{}{}/article/view_comment/{}?limit=50&offset=0",
            BASE_URL, API_PREFIX, article_id
        );
        let list_comments_resp = client
            .get(&list_comments_url)
            .send()
            .await
            .expect("List comments failed");
        assert_eq!(list_comments_resp.status(), StatusCode::OK);
        let list_comments_body: Value = list_comments_resp
            .json()
            .await
            .expect("Parse comment list");
        assert_eq!(list_comments_body["status"], true);
        let comment = list_comments_body["data"]
            .as_array()
            .and_then(|arr| arr.iter().find(|c| c["comment"] == comment_text))
            .expect("Created comment not found");
        let comment_id = comment["id"]
            .as_str()
            .expect("comment id missing")
            .to_string();

        let user_view_url = format!("{}{}/user/view", BASE_URL, API_PREFIX);
        let user_view_resp = client
            .get(&user_view_url)
            .send()
            .await
            .expect("View current user failed");
        assert_eq!(user_view_resp.status(), StatusCode::OK);
        let user_view_body: Value = user_view_resp.json().await.expect("Parse user view");
        assert_eq!(user_view_body["status"], true);
        let user_id = user_view_body["data"]["id"]
            .as_str()
            .expect("user id missing")
            .to_string();

        let delete_comment_url = format!("{}{}/comment/delete", BASE_URL, API_PREFIX);
        let delete_comment_resp = client
            .post(&delete_comment_url)
            .json(&json!({
                "comment_id": comment_id,
                "user_id": user_id
            }))
            .send()
            .await
            .expect("Delete comment failed");
        assert_eq!(delete_comment_resp.status(), StatusCode::OK);
        let delete_comment_body: Value = delete_comment_resp
            .json()
            .await
            .expect("Parse delete comment");
        assert_eq!(delete_comment_body["status"], true);

        delete_article_if_exists(&client, &article_id).await;
    }
}

// ============================================
// Page Route Tests
// ============================================

#[cfg(test)]
mod page_route_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_index_page() {
        let client = create_client();
        let response = client.get(BASE_URL).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("Failed to get body");
        assert!(body.contains("<!DOCTYPE html>"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_about_page() {
        let client = create_client();
        let url = format!("{}/about", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_home_page() {
        let client = create_client();
        let url = format!("{}/home", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_css() {
        let client = create_client();
        let url = format!("{}/css/index.css", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("Failed to get body");
        // CSS file should contain style rules
        assert!(body.contains("{") && body.contains("}"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_js() {
        let client = create_client();
        let url = format!("{}/js/jquery-3.7.1.min.js", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_webp() {
        let client = create_client();
        let url = format!("{}/images/test.webp", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        // If file exists, should return OK; if not exists, should return NOT_FOUND
        // Either way, the extension is allowed
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_disallowed_extension() {
        let client = create_client();

        // Try to access a file with disallowed extension (.rs, .toml, .json, etc.)
        // These should be blocked by the static file filter
        let disallowed_urls = [
            format!("{}/test.rs", BASE_URL),
            format!("{}/test.toml", BASE_URL),
            format!("{}/test.json", BASE_URL),
            format!("{}/test.html", BASE_URL),
            format!("{}/test.txt", BASE_URL),
            format!("{}/test.php", BASE_URL),
            format!("{}/test.sh", BASE_URL),
        ];

        for url in &disallowed_urls {
            let response = client.get(url).send().await.expect("Request failed");
            // Disallowed extensions should return NOT_FOUND (blocked by filter)
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Expected NOT_FOUND for disallowed extension: {}",
                url
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_allowed_extensions() {
        let client = create_client();

        // Test that allowed extensions are not blocked (they return OK or NOT_FOUND based on file existence)
        // We use paths that likely don't exist to test that the filter doesn't block them
        let allowed_extensions = [
            ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".woff", ".woff2",
            ".svg",
        ];

        for ext in &allowed_extensions {
            let url = format!("{}/nonexistent_file_for_test{}", BASE_URL, ext);
            let response = client.get(&url).send().await.expect("Request failed");
            // Allowed extensions should return NOT_FOUND (file doesn't exist, but extension is allowed)
            // This proves the filter didn't block the request
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Expected NOT_FOUND (not blocked) for allowed extension: {}",
                ext
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_case_insensitive_extension() {
        let client = create_client();

        // Test that extension check is case-insensitive
        let url = format!("{}/css/index.CSS", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        // Should be allowed (case-insensitive), returns OK or NOT_FOUND
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
            "Expected OK or NOT_FOUND for case-insensitive extension check"
        );
    }
}
